import { mount } from 'svelte'
import App from './App.svelte'
import './app.css'

const target = document.getElementById('app')
if (!target) {
  throw new Error('#app 挂载点不存在')
}

mount(App, { target })
