import { createApp } from 'vue';
import { Quasar, Notify, Dark } from 'quasar';
import 'quasar/src/css/index.sass';
import App from '@/App.vue';
import '@quasar/extras/material-icons/material-icons.css';
createApp(App)
    .use(Quasar, {
    plugins: {
        Notify,
        Dark
    },
    config: {},
}).mount("#tauri-app");
//# sourceMappingURL=main.js.map