import type { CommandErrorCode, TenantProfileEntry, TenantProfilePayload } from "./api";

// `loading` retient l'id demandé pour écarter la réponse tardive d'une fiche abandonnée.
// `confirmingReplacement` garde la fiche à écrire tant que l'utilisateur n'a pas répondu.
export type TenantProfileState =
  | { readonly status: "idle" }
  | { readonly status: "loading"; readonly id: string }
  | { readonly status: "saving" }
  | { readonly status: "confirmingReplacement"; readonly profile: TenantProfilePayload }
  | { readonly status: "saved"; readonly entry: TenantProfileEntry }
  | { readonly status: "failed"; readonly code: CommandErrorCode };
