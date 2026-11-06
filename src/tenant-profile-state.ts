import type { CommandErrorCode, TenantProfileEntry, TenantProfilePayload } from "./api";

// `loading` retient l'id demandé pour écarter la réponse tardive d'une fiche abandonnée.
// `confirmingReplacement` garde la fiche à écrire tant que l'utilisateur n'a pas répondu.
// `saveRejected` et `savedUnlisted` ont leur propre message : le libellé générique de leur
// code d'erreur parlerait de la quittance ou d'une fiche inutilisable.
export type TenantProfileState =
  | { readonly status: "idle" }
  | { readonly status: "loading"; readonly id: string }
  | { readonly status: "saving" }
  | { readonly status: "confirmingReplacement"; readonly profile: TenantProfilePayload }
  | { readonly status: "saved"; readonly entry: TenantProfileEntry }
  | { readonly status: "saveRejected" }
  | { readonly status: "savedUnlisted"; readonly entry: TenantProfileEntry }
  | { readonly status: "failed"; readonly code: CommandErrorCode };
