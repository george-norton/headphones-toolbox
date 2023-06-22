<script setup>
import GraphVue from './components/Graph.vue'
import FilterCardVue from './components/FilterCard.vue'
import PreProcessingCardVue from './components/PreProcessingCard.vue'
import CodecCardVue from './components/CodecCard.vue'
import AboutDialogVue from './components/AboutDialog.vue'
import InfoMenuVue from './components/InfoMenu.vue'
import { appWindow } from '@tauri-apps/api/window'
import { createDir, readTextFile, writeTextFile, BaseDirectory } from "@tauri-apps/api/fs"
import { useQuasar } from 'quasar'

const $q = useQuasar()
$q.dark.set("auto")
const about = ref(null)
const importFile = ref(null)
</script>

<script>
import { ref, reactive, toRaw } from 'vue'
import { exportFile, getCssVar } from 'quasar'
import { invoke } from '@tauri-apps/api'
import { resolveResource } from '@tauri-apps/api/path'
import { getVersion } from '@tauri-apps/api/app';
import debounce from 'lodash.debounce'

const API_VERSION = 1;
var deviceNames = { "none": "No device detected" }
var deviceListKey = ref(0)
var popup = ref(undefined)
const defaultState = { "expanded": [true, true, true] }

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
      this.validated = false
      if (this.connected) {
        this.sendState()
        invoke("read_version_info").then((version) => {
          this.versions = { ...JSON.parse(version), ...{ "serial_number": this.device, "client_api_version": API_VERSION } }
          if (version.minimum_supported_version > API_VERSION) {
            this.$q.notify({ type: 'negative', message: "Fimrware is too new, this version of Ploopy Headphones Toolkit is not supported." })
          }
          else if (API_VERSION > version.current_version) {
            this.$q.notify({ type: 'negative', message: "Firmware is too old, this version of Ploopy Headphones Toolkit is not supported." })
          }
          else {
            this.validated = true
          }
        })
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
      const data = reader.readAsText(this.file);
      reader.onload = () => {
        try {
          const configData = JSON.parse(reader.result);
          this.migrateConfig(configData)

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
      connected: ref(undefined),
      validated: ref(undefined),
      versions: ref(undefined)
    }
  },
  components: {
    FilterCardVue,
    PreProcessingCardVue,
    CodecCardVue,
    AboutDialogVue,
    InfoMenuVue
  },
  methods: {
    migrateConfig(config) {
      if (!("state" in config)) {
        config.state = structuredClone(defaultState)
      }
      if (!("codec" in config)) {
        config.codec = { "oversampling": false, "phase": false, "rolloff": false, "de_emphasis": false }
      }
      else {
        // Initially these we integers not booleans, so convert them.
        config.codec.oversampling = config.codec.oversampling != 0
        config.codec.phase = config.codec.phase != 0
        config.codec.rolloff = config.codec.rolloff != 0
        config.codec.de_emphasis = config.codec.de_emphasis != 0
      }
      for (var i in config.filters) {
        if (!("a0" in config.filters[i])) {
          config.filters[i] = { ...config.filters[i], ...{ a0: 0, a1: 0, a2: 0, b0: 0, b1: 0, b2: 0 } }
        }
      }
      if ("reverseStereo" in config.preprocessing) {
        config.preprocessing.reverse_stereo = config.preprocessing.reverseStereo
        delete config.preprocessing.reverseStereo
      }
    },
    pageHeight(offset) {
      const height = offset ? `calc(100vh - ${offset}px)` : '100vh'
      return { height: height }
    },
    readDeviceConfiguration() {
      invoke("load_config").then((deviceConfig) => {
        var config = JSON.parse(deviceConfig)
        config.id = this.tab
        config.name = this.tabs[this.tab].name
        config.state = structuredClone(toRaw(this.tabs[this.tab].state))
        this.tabs[this.tab] = config
      })
    },
    readDefaultConfiguration() {
      resolveResource('resources/configuration.json').then((configJson) =>
        readTextFile(configJson).then((defaultConfiguration) => {
          var config = JSON.parse(defaultConfiguration)
          config.id = this.tab
          config.name = this.tabs[this.tab].name
          config.state = structuredClone(toRaw(this.tabs[this.tab].state))
          this.tabs[this.tab] = config
        }))
    },
    addConfiguration() {
      var nextId = this.tabs.length
      // Try not to make any changes to the sound on the connected headphones
      // First try to clone the current config - if it exists
      if (this.tabs.length > 0) {
        var config = structuredClone(toRaw(this.tabs[this.tab]))
        config.id = nextId
        config.name = "Unnamed configuration"
        config.state = structuredClone(defaultState)
        this.tabs.push(config)
        this.tab = nextId
        return;
      }
      invoke("load_config").then((deviceConfig) => {
        var config = JSON.parse(deviceConfig)
        config.name = "Unnamed configuration"
        config.id = nextId
        config.state = structuredClone(defaultState)
        this.tabs.push(config)
        this.tab = nextId
      }).catch(err => {
        resolveResource('resources/configuration.json').then((configJson) =>
          readTextFile(configJson).then((defaultConfiguration) => {
            var config = JSON.parse(defaultConfiguration)
            config.id = nextId
            config.state = structuredClone(defaultState)
            this.tabs.push(config)
            this.tab = nextId
          }))
      })
    },
    deleteConfiguration() {
      var id = 0;
      for (var i = 0; i < this.tabs.length; i++) {
        if (this.tabs[i].id == this.tab) {
          this.tabs.splice(i, 1)
          this.tab = Math.min(id, this.tabs.length - 1)
          i--
        }
        else {
          this.tabs[i].id = id
          id++
        }
      }
    },
    sendState: debounce(function () {
      if (this.validated && this.tab !== undefined && this.tabs[this.tab] !== undefined) {
        var sendConfig = {
          "preprocessing": { "preamp": this.tabs[this.tab].preprocessing.preamp, "reverse_stereo": this.tabs[this.tab].preprocessing.reverse_stereo },
          "filters": this.tabs[this.tab].filters,
          "codec": this.tabs[this.tab].codec
        }
        console.log(this.versions)
        if (!("current_version" in this.versions) || this.versions.current_version < 2) {
          console.log(sendConfig)
          for (var f in sendConfig.filters) {
            console.log(f)
            if (sendConfig.filters[f].filter_type == "custom_iir" && sendConfig.filters[f].enabled) {
              this.$q.notify({ type: 'negative', message: "IIR filters are not supported by this firmware version." })
              break
            }
          }
        }
        invoke('write_config', { config: JSON.stringify(sendConfig) }).then((message) => {
        })
      }
    }, 5),
    saveState: debounce(function () {
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
    }, 100),
    loadState() {
      readTextFile(
        "configuration.json",
        { dir: BaseDirectory.AppData }
      ).then((response) => {
        var config = JSON.parse(response)
        if (config) {
          for (var c in config.configurations) {
            this.migrateConfig(config.configurations[c])
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
      const exportData = structuredClone(toRaw(this.tabs[this.tab]))
      exportData.version = this.version
      delete exportData.state
      const config = JSON.stringify(exportData, null, 4)
      exportFile(this.tabs[this.tab].name + ".json", config)
    },
    importConfiguration(){
      importFile.pickFiles()
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
    <q-header elevated class="top-bar">

      <q-bar data-tauri-drag-region class="title-bar">
        <q-icon style="pointer-events: none;" name="img:ploopy.png" />
        <div style="pointer-events: none;">Ploopy Headphones Toolbox</div>
        <q-space />
        <q-btn dense flat icon="minimize" @click="appWindow.minimize()" />
        <q-btn dense flat icon="crop_square" @click="appWindow.maximize()" />
        <q-btn dense flat icon="close" @click="appWindow.close()" />
      </q-bar>

      <q-toolbar class="text-white justify-start">
        <q-select borderless dark :options-dark="$q.dark.isActive" v-model="device" :key="deviceListKey"
          :options="devices" option-value="value" :option-label="item => deviceNames[item]" map-options
          ref="deviceSelect">
          <template v-slot:prepend>
            <q-icon name="headphones" />
          </template>
        </q-select>

        <InfoMenuVue :disable="!connected" v-bind:versions="versions" />
        <q-btn flat dense icon="edit" :disable="!connected">
          <q-tooltip>
            Rename this device.
          </q-tooltip>
          <q-popup-edit v-model="popup" v-slot="devicePopup" anchor="center middle" self="top middle" :cover=false>
            <q-input v-model="deviceNames[device]" dense autofocus @keyup.enter="devicePopup.set"
              @focus="(input) => input.target.select()" @update:model-value="$value => updateDeviceName($value)" />
          </q-popup-edit>
        </q-btn>
        <q-space />
        <q-btn flat dense icon="save_alt" :disable="!validated" @click="invoke('save_config')">
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
              <q-item clickable v-close-popup :disable="!validated" @click="invoke('factory_reset')">
                <q-item-section>Erase saved configuration</q-item-section>
              </q-item>
              <q-separator />
              <q-item clickable v-close-popup @click="about.show()">
                <q-item-section>About</q-item-section>
              </q-item>
            </q-list>
          </q-menu>
        </q-btn>
      </q-toolbar>

      <AboutDialogVue ref="about" />
      <q-file ref="importFile" class="hidden" accept=".json" clearable filled v-model="file" />
    </q-header>

    <q-page-container>
      
      <q-page :style-fn="pageHeight" class="scroll overflow-auto">

        <q-tabs v-model="tab" dense align="left" :breakpoint="0">
          <q-tab v-for="t in tabs" :name="t.id" :label="t.name" />
          <q-btn flat dense icon="add" @click="$event => addConfiguration()">
            <q-tooltip>
              Add a new configuration.
            </q-tooltip>
          </q-btn>

          <q-separator vertical inset />

          <q-btn flat dense icon="edit">
            <q-tooltip>
              Rename this configuration.
            </q-tooltip>
            <q-popup-edit auto-save v-model="popup" v-slot="tabPopup" anchor="center middle" self="top middle"
              :cover=false>
              <q-input v-model="tabs[tab].name" dense autofocus @keyup.enter="tabPopup.set"
                @focus="(input) => input.target.select()" />
            </q-popup-edit>
          </q-btn>
          <q-btn flat dense icon="delete" @click="$event => deleteConfiguration()">
            <q-tooltip>
              Delete this configuration.
            </q-tooltip>
          </q-btn>
          <q-space />
          <q-btn flat dense icon="more_vert">
            <q-menu>
              <q-list style="min-width: 14em">
                <q-item clickable v-close-popup @click="exportConfiguration()">
                  <q-item-section>Export to JSON</q-item-section>
                </q-item>
                <q-item clickable v-close-popup @click="importFile.pickFiles()">
                  <q-item-section>Import from JSON</q-item-section>
                </q-item>
                <q-item clickable v-close-popup :disable="!validated" @click="readDeviceConfiguration()">
                  <q-item-section>Read config from device</q-item-section>
                </q-item>
                <q-item clickable v-close-popup @click="readDefaultConfiguration()">
                  <q-item-section>Reset config to default</q-item-section>
                </q-item>
              </q-list>
            </q-menu>
          </q-btn>
        </q-tabs>

        <q-tab-panels v-model="tab" animated>
          <q-tab-panel v-for="t in tabs" :name="t.id" class="panel">
            <div class="column q-gutter-md q-ma-none">
              <PreProcessingCardVue v-model:preamp="t.preprocessing.preamp"
                v-model:reverse_stereo="t.preprocessing.reverse_stereo" v-model:expansion="t.state.expanded[0]" />
              <FilterCardVue v-model:filters="t.filters" v-model:expansion="t.state.expanded[1]" />
              <CodecCardVue v-model:oversampling="t.codec.oversampling" v-model:phase="t.codec.phase"
                v-model:rolloff="t.codec.rolloff" v-model:de_emphasis="t.codec.de_emphasis"
                v-model:expansion="t.state.expanded[2]" />
            </div>
          </q-tab-panel>
        </q-tab-panels>

      </q-page>
    </q-page-container>

    <q-footer elevated>
      <div class="block full-width">
        <GraphVue ref="graph" v-model:filters="tabs[tab]" />
      </div>

    </q-footer>

  </q-layout>
</template>

 
