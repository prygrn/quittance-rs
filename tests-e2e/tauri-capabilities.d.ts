// Capacité propre à tauri-driver : le binaire de l'app à lancer pour chaque session.
declare global {
  namespace WebdriverIO {
    interface Capabilities {
      "tauri:options"?: {
        readonly application: string;
      };
    }
  }
}

export {};
