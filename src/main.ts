import { createApp } from 'vue'
import { Quasar, Notify } from 'quasar'
import 'quasar/src/css/index.sass'
import App from '@/App.vue'
import '@quasar/extras/material-icons/material-icons.css'

createApp(App)
    .use(Quasar, {
        plugins: {
            Notify
        },
        config: {
            notify: {
                position: 'bottom'
            },
        },
    }).mount("#tauri-app")
