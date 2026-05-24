import { mount } from 'svelte';
import SettingsApp from './settings-app.svelte';
import './app.css';

const app = mount(SettingsApp, {
  target: document.getElementById('app')!,
});

export default app;
