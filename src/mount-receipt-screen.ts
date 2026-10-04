import type { ReceiptGateway } from "./receipt-gateway";

/**
 * Branche l'écran de quittance sur le balisage de `index.html` présent dans `root`,
 * puis charge la liste des modèles. La promesse se résout une fois cette liste affichée.
 */
export function mountReceiptScreen(options: {
  readonly root: ParentNode;
  readonly gateway: ReceiptGateway;
}): Promise<void> {
  void options;
  return Promise.reject(new Error("not implemented"));
}
