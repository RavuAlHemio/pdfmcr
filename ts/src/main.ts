import { Splitter } from "./splitter.js";

// "globals are evil"
declare global {
    interface Window { PdfMcr: any; }
}
window.PdfMcr = {
    Splitter: Splitter,
};
