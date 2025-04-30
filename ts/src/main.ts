"use strict";

import { Annotations } from "./annotations";
import { Serialize } from "./serialize";
import { Splitter } from "./splitter";
import { SvgDrag } from "./svgdrag";
import { TextManagement } from "./textmgmt";

// "globals are evil"
declare global {
    interface Window { PdfMcr: any; }
}
window.PdfMcr = {
    Annotations: Annotations,
    Serialize: Serialize,
    Splitter: Splitter,
    SvgDrag: SvgDrag,
    TextManagement: TextManagement,
    init: function () {
        window.PdfMcr.Serialize.init();
        window.PdfMcr.Splitter.init();
        window.PdfMcr.SvgDrag.init();
        window.PdfMcr.TextManagement.init();
    }
};
