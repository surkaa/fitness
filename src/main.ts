import {createApp} from "vue";
import App from "./App.vue";
import router from "./router";
import {Dialog, Notify, Quasar} from 'quasar'
import '@quasar/extras/material-icons/material-icons.css'
import 'quasar/src/css/index.sass'
import './style.css'

createApp(App)
    .use(router)
    .use(Quasar, {
        plugins: {
            Notify,
            Dialog
        },
    })
    .mount("#app");
