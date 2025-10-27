export function construct(
  tag,
  superclass,
  superclassTag,
  constructor,
  template,
  observedAttributes,
) {
  customElements.define(
    tag,
    class extends superclass {
      static get observedAttributes() {
        return observedAttributes;
      }

      constructor() {
        super();

        constructor(this);
      }

      attributeChangedCallback(name, oldValue, newValue) {
        this._attributeChangedCallback(this, name, oldValue, newValue);
      }

      connectedCallback() {
        const shadowMode = this._attachShadow(this);
        if (shadowMode) {
          this.attachShadow({ mode: shadowMode });
        }
        this.insertAdjacentHTML("afterbegin", template);
        this._connectedCallback(this);
      }

      disconnectedCallback() {
        this._disconnectedCallback(this);
      }

      adoptedCallback() {
        this._adoptedCallback(this);
      }
    },
    superclassTag ? { extends: superclassTag } : undefined,
  );
}
