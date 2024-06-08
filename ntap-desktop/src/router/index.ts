import { createRouter, createWebHistory } from 'vue-router';
import Overview from '../components/Overview.vue';
import RemoteAddress from '../components/RemoteAddress.vue';
import Socket from '../components/Socket.vue';
import Process from '../components/Process.vue';
import Interface from '../components/Interface.vue';
import NetRoute from '../components/NetRoute.vue';

const routes = [
  {
    path: '/',
    name: 'Overview',
    component: Overview,
  },
  {
    path: '/overview',
    name: 'Overview2',
    component: Overview,
  },
  {
    path: '/remote',
    name: 'RemoteAddress',
    component: RemoteAddress,
  },
  {
    path: '/socket',
    name: 'Socket',
    component: Socket,
  },
  {
    path: '/process',
    name: 'Process',
    component: Process,
  },
  {
    path: '/interface',
    name: 'Interface',
    component: Interface,
  },
  {
    path: '/netroute',
    name: 'Route',
    component: NetRoute,
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
