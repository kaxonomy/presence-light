import { mount } from 'svelte';
import { invoke, isTauri } from '@tauri-apps/api/core';
import App from './App.svelte';
import './styles.css';

if (isTauri()) {
  for (const level of ['debug', 'info', 'warn', 'error'] as const) {
    const output = console[level].bind(console);
    console[level] = (...values: unknown[]) => {
      output(...values);
      const message = values
        .map((value) => {
          if (value instanceof Error) return value.stack ?? value.message;
          if (typeof value === 'string') return value;
          try {
            return JSON.stringify(value);
          } catch {
            return String(value);
          }
        })
        .join(' ');
      void invoke('debug_log', { level: level.toUpperCase(), message }).catch(() => {});
    };
  }
}

mount(App, { target: document.getElementById('app')! });
