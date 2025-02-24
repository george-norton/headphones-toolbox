<script>
import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-shell';
import { getTauriVersion, getVersion } from '@tauri-apps/api/app';
import { arch, version, platform } from '@tauri-apps/plugin-os';

export default {
    setup() {
        const dialog = ref(false)
        const position = ref('top')

        return {
            dialog,
            position,

            show(pos) {
                position.value = pos
                dialog.value = true
            },
            async openURL(url) {
                await open(url);
            }
        }
    },
    mounted() {
        getVersion().then((version) => this.version = version)
        getTauriVersion().then((tauriVersion) => this.tauriVersion = tauriVersion)
        this.archName = arch()
        this.platformName = platform()
        this.osVersion = version()
    },
    data() {
        return {
            version: "Unknown",
            tauriVersion: "Unknown",
            archName: "Unknown",
            platformName: "Unknown",
            osVersion: "Unknown"
        }
    }
}
</script>
<template>
    <q-dialog v-model="dialog" :position="position">
        <q-card style="width: 30em">
            <q-card-section class="row items-center no-wrap about-top q-py-sm q-gutter-sm">
                <q-icon style="pointer-events: none;" size="md" name="img:ploopy.png" />
                <div class="">Ploopy Headphones Toolkit</div>
                <q-space />
                <q-btn dense flat icon="close" v-close-popup>
                    <q-tooltip class="bg-white text-primary">Close</q-tooltip>
                </q-btn>
            </q-card-section>

            <q-card-section class="col items-center no-wrap q-py-sm">
                <div class="bold-heading">Build info</div>
                <div class="col q-gutter-none justify-start q-pl-lg">
                    <div>Application Version: {{ version }}</div>
                    <div>Tauri Version: {{ tauriVersion }}</div>
                    <div>Platform: {{ platformName }}</div>
                    <div>OS Version: {{ osVersion }}</div>
                    <div>Architecture: {{ archName }}</div>
                </div>
            </q-card-section>
            <q-card-section class="col items-center no-wrap q-pt-sm  q-pb-lg">
                <div class="bold-heading">Useful links</div>
                <div class="col q-gutter-none justify-start q-pl-lg">
                    <div class="row q-gutter-sm justify-start items-center cursor-pointer"
                        @click="openURL('https://github.com/george-norton/headphones-toolbox')">
                        <div>Get the source code from github</div>
                    </div>
                    <div class="row q-gutter-sm justify-start items-center cursor-pointer"
                        @click="openURL('https://github.com/ploopyco/headphones')">
                        <div>Get firmware from github</div>
                    </div>
                    <div class="row q-gutter-sm justify-start items-center cursor-pointer"
                        @click="openURL('https://ploopy.co')">
                        <div>About Ploopy Headphones</div>
                    </div>
                </div>
            </q-card-section>
            <q-card-section
                class="row justify-between items-center no-wrap top-bar q-pt-none q-pb-sm q-gutter-sm about-bottom">
                <div class="text-subtitle2">by George Norton</div>
                <div class="text-subtitle2 cursor-pointer"
                    @click="openURL('https://www.gnu.org/licenses/gpl-3.0.en.html#license-text')">
                    Licensed under the GPL-v3.0
                </div>
            </q-card-section>
        </q-card>
    </q-dialog>
</template>