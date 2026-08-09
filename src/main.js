import ui from '@nuxt/ui/vue-plugin';
import { createApp } from 'vue';

import App from './App.vue';
import router from './router';
import './styles/main.css';

const app = createApp( App );

app.use( router );
app.use( ui );

router.isReady().then( () => {
  app.mount( '#app' );
});
