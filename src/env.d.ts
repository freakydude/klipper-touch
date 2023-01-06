/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_MOONRAKER_WEBSOCKET: string;
  readonly VITE_MOONRAKER_API: string;
  // more env variables...
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
