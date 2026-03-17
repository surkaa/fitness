import {createApp} from "vue";
import App from "./App.vue";
import router from "./router";
import {Dialog, Notify, Quasar} from 'quasar'
import '@quasar/extras/material-icons/material-icons.css'
import 'quasar/src/css/index.sass'
import './style.css'
import {createPinia} from "pinia";

createApp(App)
    .use(createPinia())
    .use(router)
    .use(Quasar, {
        plugins: {
            Notify,
            Dialog
        },
    })
    .mount("#app");
