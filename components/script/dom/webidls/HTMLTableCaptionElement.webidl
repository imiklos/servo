/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// https://html.spec.whatwg.org/multipage/#htmltablecaptionelement
[Exposed=Window, HTMLConstructor]
interface HTMLTableCaptionElement : HTMLElement {
  // also has obsolete members
};

// https://html.spec.whatwg.org/multipage/#HTMLTableCaptionElement-partial
partial interface HTMLTableCaptionElement {
  // [CEReactions]
  //          attribute DOMString align;
};
