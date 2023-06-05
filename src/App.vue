<script setup>
import GraphVue from './components/Graph.vue'
import FilterCardVue from './components/FilterCard.vue'
import PreProcessingCardVue from './components/PreProcessingCard.vue'
import CodecCardVue from './components/CodecCard.vue'
import { appWindow } from '@tauri-apps/api/window'
import { createDir, readTextFile, writeTextFile, BaseDirectory } from "@tauri-apps/api/fs";

</script>

<script>
import { ref, reactive } from 'vue'
import { exportFile, getCssVar } from 'quasar'
import { invoke } from '@tauri-apps/api'
import { resolveResource } from '@tauri-apps/api/path'
import { getVersion } from '@tauri-apps/api/app';
import debounce from 'lodash.debounce'

var idSequence = 0
var deviceNames = { "none": "No device detected" }
var deviceListKey = ref(0)
var popup = ref(undefined)

export default {
  setup() {

  },
  mounted() {
    getVersion().then((version) => this.version = version)
    this.loadState()
    this.pollDevices()
    window.setInterval(this.pollDevices, 5000)
  },
  unmounted() {
    this.saveState()
  },
  watch: {
    device() {
      this.openDevice()
    },
    connected() {
      if (this.connected) {
        this.sendState()
      }
    },
    tab() {
      this.sendState()
      this.saveState()
    },
    tabs: {
      handler() {
        this.sendState()
        this.saveState()
      },
      deep: true
    },
    file() {
      const reader = new FileReader();
      console.log(this.file);
      const data = reader.readAsText(this.file);
      reader.onload = () => {
        try {
          console.log(reader.result)
          const configData = JSON.parse(reader.result);
          var nextId = this.tabs.length
          configData.id = nextId
          if (configData.name && configData.filters) {
            this.tabs.push(configData)
            this.tab = nextId
          }
          else {
            throw new SyntaxError("Missing JSON elements");
          }
        } catch (err) {
          this.$q.notify({ type: 'negative', message: "Failed to load config" })
          console.log(err)
        }
      };
    }
  },
  data() {
    return {
      file: ref(undefined),
      tab: ref(0),
      tabs: reactive([]),
      devices: reactive([]),
      deviceOptions: reactive([]),
      device: ref(undefined),
      connected: ref(undefined)
    }
  },
  components: {
    FilterCardVue,
    PreProcessingCardVue,
    CodecCardVue
  },
  methods: {
    pageHeight(offset) {
      const height = offset ? `calc(100vh - ${offset}px)` : '100vh'
      return { height: height }
    },
    addConfiguration() {
      resolveResource('resources/configuration.json').then((configJson) =>
        readTextFile(configJson).then((defaultConfiguration) => {
          var nextId = this.tabs.length
          var config = JSON.parse(defaultConfiguration)
          config.id = nextId
          this.tabs.push(config)
          this.tab = nextId
        }))


    },
    deleteConfiguration() {
      for (var i = 0; i < this.tabs.length; i++) {
        this.tabs[i].id = i
        if (this.tabs[i].id == this.tab) {
          this.tabs.splice(i, 1)
          if (i > 0) {
            this.tab = this.tabs[i - 1].id
          }
          else if (i < this.tabs.length) {
            this.tab = this.tabs[i].id
          }
        }
      }
    },
    sendState() {
      if (this.tab) {
        var sendConfig = {
          "preprocessing": { "preamp": this.tabs[this.tab].preprocessing.preamp / 100, "reverse_stereo": this.tabs[this.tab].preprocessing.reverseStereo },
          "filters": this.tabs[this.tab].filters
        }
        invoke('write_config', { config: JSON.stringify(sendConfig) }).then((message) => {
        })
      }
    },
    saveState: debounce(function() {
      var config = {
        "currentConfiguration": this.tab,
        "configurations": this.tabs,
        "deviceNames": deviceNames,
        "version": this.version
      }
      try {
        createDir("", { dir: BaseDirectory.AppData, recursive: true }).then(
        writeTextFile(
          {
            contents: JSON.stringify(config, null, 4),
            path: "configuration.json"
          },
          { dir: BaseDirectory.AppData }
        ))
      } catch (e) {
        console.log(e);
      }
    }, 1000),
    loadState() {
      readTextFile(
        "configuration.json",
        { dir: BaseDirectory.AppData }
      ).then((response) => {
        var config = JSON.parse(response)
        if (config) {
          for (var c in config.configurations) {
            if (config.configurations[c].id == config.currentConfiguration) {
              this.tab = c
            }
            config.configurations[c].id = c
          }
          this.tabs = reactive(config.configurations)
          deviceNames = config.deviceNames
        }
      })
        .catch((error) => {
          console.error(error);
        });
    },
    async exportConfiguration() {
      const exportData = this.tabs[this.tab]
      exportData.version = this.version
      const config = JSON.stringify(exportData, null, 4)
      exportFile(this.tabs[this.tab].name + ".json", config)
    },
    importConfiguration() {
      this.$refs.importFile.pickFiles()
    },
    updateDeviceName(name) {
      deviceNames[this.device] = name
      deviceListKey.value += 1
      this.saveState()
    },
    openDevice() {
      invoke('open', { serialNumber: this.device }).then((result) => {
        if (result) {
          this.$q.notify({ type: 'positive', message: "Device connected" })
          this.connected = true
        }
      })
    },
    pollDevices() {
      invoke('poll_devices').then((message) => {
        var devices = JSON.parse(message)
        for (var d in devices) {
          if (!(devices[d] in deviceNames)) {
            if (devices.length == 1 && !("Ploopy Headphones" in deviceNames)) {
              // Most people will only have one device, so use a friendly name
              deviceNames[devices[d]] = "Ploopy Headphones"
            }
            else {
              deviceNames[devices[d]] = "Headphones [" + devices[d] + "]"
            }
          }
        }
        Object.assign(this.devices, devices)

        if ((this.device == undefined || this.device == "none") && this.devices.length > 0) {
          this.device = this.devices[0];
        }
        else {
          if (this.device == undefined && this.devices.length == 0) {
            this.$q.notify({ type: 'negative', message: "No devices detected" })
            this.device = "none"
            this.connected = false
          }
          else if (this.device != "none") {
            if (this.connected && (devices.indexOf(this.device) == -1)) {
              this.$q.notify({ type: 'negative', message: "Device disconnected" })
              this.connected = false
            }
            else if (!this.connected) {
              if (devices.indexOf(this.device) != -1) {
                this.openDevice()
              }
            }
          }
        }
      })
    }
  }
}
</script>
<template>
  <q-layout view="hHh lpR fFf">
    <q-header elevated class="bg-primary text-white">

      <q-bar data-tauri-drag-region>
        <q-icon style="pointer-events: none;" name="img:ploopy.png" />
        <div style="pointer-events: none;">Ploopy Headphones Toolbox</div>
        <q-space />
        <q-btn dense flat icon="minimize" @click="appWindow.minimize()" />
        <q-btn dense flat icon="crop_square" @click="appWindow.maximize()" />
        <q-btn dense flat icon="close" @click="appWindow.close()" />
      </q-bar>

      <q-toolbar class="bg-primary text-white justify-start">
        <q-select filled v-model="device" :key="deviceListKey" :options="devices" option-value="value"
          :option-label="item => deviceNames[item]" map-options dark :options-dark=false bg-color="primary"
          ref="deviceSelect">
          <template v-slot:prepend>
            <q-icon name="headphones" />
          </template>
        </q-select>
        <q-btn flat dense icon="edit" :disable="!connected">
          <q-tooltip>
            Rename this device.
          </q-tooltip>
          <q-popup-edit v-model="popup" v-slot="devicePopup" anchor="center middle" self="top middle" :cover=false>
            <q-input v-model="deviceNames[device]" dense autofocus @keyup.enter="devicePopup.set"
              @focus="(input) => input.target.select()" @update:model-value="$value => updateDeviceName($value)" />
          </q-popup-edit>
        </q-btn>
        <q-btn flat dense icon="restart_alt" :disable="!connected" @click="invoke('reboot_bootloader')" class="hidden">
          <q-tooltip>
            Reboot this device into the bootloader so you can install new firmware.
          </q-tooltip>
        </q-btn>
        <q-btn flat dense icon="delete" :disable="!connected" @click="invoke('factory_reset')" class="hidden">
          <q-tooltip>
            Reset the device to its factory default settings.
          </q-tooltip>
        </q-btn>
        <q-space />
        <q-btn flat dense icon="save_alt" :disable="!connected" @click="invoke('save_config')">
          <q-tooltip>
            Persist the current configuration to flash memory on the DAC.
          </q-tooltip>
        </q-btn>
        <q-btn flat dense icon="more_vert">
          <q-menu>
            <q-list style="min-width: 16em">
              <q-item clickable v-close-popup :disable="!connected" @click="invoke('reboot_bootloader')">
                <q-item-section>Reboot into bootloader</q-item-section>
              </q-item>
              <q-item clickable v-close-popup :disable="!connected">
                <q-item-section>Erase saved configuration</q-item-section>
              </q-item>
            </q-list>
          </q-menu>
        </q-btn>
      </q-toolbar>

    </q-header>

    <q-page-container>
      <q-page :style-fn="pageHeight" class="scroll overflow-auto">

        <q-tabs v-model="tab" dense align="left" :breakpoint="0" class="bg-grey-1 text-black" ref="tabs">
          <q-tab v-for="t in tabs" :name="t.id" :label="t.name" />
          <q-btn flat dense icon="add" text-color="grey-9" @click="$event => addConfiguration()">
            <q-tooltip>
              Add a new configuration.
            </q-tooltip>
          </q-btn>

          <q-separator vertical inset />

          <q-btn flat dense icon="edit" text-color="grey-9">
            <q-tooltip>
              Rename this configuration.
            </q-tooltip>
            <q-popup-edit auto-save v-model="popup" v-slot="tabPopup" anchor="center middle" self="top middle" :cover=false>
              <q-input v-model="tabs[tab].name" dense autofocus @keyup.enter="tabPopup.set"
                @focus="(input) => input.target.select()" />
            </q-popup-edit>
          </q-btn>
          <q-btn flat dense icon="delete" text-color="grey-9" @click="$event => deleteConfiguration()">
            <q-tooltip>
              Delete this configuration.
            </q-tooltip>
          </q-btn>

          <q-space />

          <q-btn flat dense icon="file_download" text-color="grey-9" @click="importConfiguration()">
            <q-tooltip>
              Import a configuration from a JSON file.
            </q-tooltip>
            <q-file ref="importFile" class="hidden" accept=".json" clearable filled v-model="file" />
          </q-btn>
          <q-btn flat dense icon="file_upload" text-color="grey-9" @click="exportConfiguration()">
            <q-tooltip>
              Export this configuration to a JSON file.
            </q-tooltip>
          </q-btn>
          <q-btn flat dense icon="usb" text-color="grey-9" @click="" class="hidden">
            <q-tooltip>
              Import configuration from the connected device.
            </q-tooltip>
          </q-btn>
          <q-btn flat dense icon="more_vert" text-color="grey-9">
            <q-menu>
              <q-list style="min-width: 100px">
                <q-item clickable v-close-popup>
                  <q-item-section>Export to JSON</q-item-section>
                </q-item>
                <q-item clickable v-close-popup>
                  <q-item-section>Import from JSON</q-item-section>
                </q-item>
                <!--q-item clickable v-close-popup>
                        <q-item-section>Import from device</q-item-section>
                      </q-item-->
              </q-list>
            </q-menu>
          </q-btn>

        </q-tabs>
        <q-tab-panels v-model="tab" animated class="bg-grey-1">
          <q-tab-panel v-for="t in tabs" :name="t.id" class="column q-gutter-md q-ma-none bg-grey-1">
            <PreProcessingCardVue v-model:preamp="t.preprocessing.preamp" v-model:reverseStereo="t.preprocessing.reverseStereo" />
            <FilterCardVue v-model:filters="t.filters" />
            <CodecCardVue />
          </q-tab-panel>
        </q-tab-panels>

      </q-page>
    </q-page-container>

    <q-footer elevated class="bg-grey-8 text-white">

      <div class="block full-width bg-white">
        <GraphVue ref="graph" v-model:filters="tabs[tab]" />
      </div>

    </q-footer>

  </q-layout>
</template>

 
