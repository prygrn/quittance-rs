/**
 * Remplace le texte d'un élément seulement s'il change : réécrire un texte identique ferait
 * répéter les régions `aria-live` par certains lecteurs d'écran.
 */
export function setText(options: { readonly element: HTMLElement; readonly text: string }): void {
  if (options.element.textContent !== options.text) {
    options.element.textContent = options.text;
  }
}
