import type { CommandErrorCode, TemplateInfo } from "./api";
import type { ReceiptFormErrors } from "./receipt-form";
import type { ScreenState } from "./screen-state";
import type { TenantProfileState } from "./tenant-profile-state";

/** Action dont la validation produit les erreurs affichées sous les champs. */
export type ValidatedAction = "preview" | "saveTenantProfile";

/** État complet de l'écran : la machine `ScreenState` et ce que seule l'UI doit retenir. */
export interface ReceiptScreenModel {
  readonly screenState: ScreenState;
  /** Erreurs affichées sous les champs ; `null` hors d'une tentative d'aperçu refusée. */
  readonly fieldErrors: ReceiptFormErrors | null;
  /** Les erreurs d'une fiche ignorent la période et la date de paiement. */
  readonly validatedAction: ValidatedAction;
  /** Vrai quand la saisie a changé après un aperçu : `idle` n'en garde pas la trace. */
  readonly isPreviewStale: boolean;
  readonly templates: readonly TemplateInfo[];
  /** Échec du chargement des modèles, qui rend tout aperçu impossible. */
  readonly templateLoadError: CommandErrorCode | null;
  readonly tenantProfileState: TenantProfileState;
}
