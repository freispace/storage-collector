import { mount } from 'svelte';
import LogsApp from './logs-app.svelte';
import './app.css';

const app = mount(LogsApp, {
  target: document.getElementById('app')!,
});

export default app;
