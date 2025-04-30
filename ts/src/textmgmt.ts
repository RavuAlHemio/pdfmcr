import { childElementsNamedNS, pointsValue, SVG_NS } from './common';
import { Annotations } from './annotations';


export namespace TextManagement {
    interface TextForm {
        editLabelSection: HTMLDivElement,

        textSpanSelect: HTMLSelectElement,

        textArea: HTMLTextAreaElement,
        fontBoldCheckbox: HTMLInputElement,
        fontItalicCheckbox: HTMLInputElement,
        fontSizeInput: HTMLInputElement,
        charSpacingInput: HTMLInputElement,
        wordSpacingInput: HTMLInputElement,
        leadingInput: HTMLInputElement,

        languageEnabledCheckbox: HTMLInputElement,
        languageInput: HTMLInputElement,
        altTextEnabledCheckbox: HTMLInputElement,
        altTextInput: HTMLTextAreaElement,
        actualTextEnabledCheckbox: HTMLInputElement,
        actualTextInput: HTMLTextAreaElement,
        expansionEnabledCheckbox: HTMLInputElement,
        expansionTextInput: HTMLTextAreaElement,
    }

    let textForm: TextForm|null = null;
    let selectedText: SVGTextElement|null = null;
    let selectedTextSpan: SVGTSpanElement|null = null;

    export function textSelected(newlySelectedText: SVGTextElement): void {
        if (textForm === null) {
            return;
        }

        selectedText = newlySelectedText;

        // repopulate list of tspans
        while (textForm.textSpanSelect.options.length > 0) {
            textForm.textSpanSelect.options.remove(textForm.textSpanSelect.options.length - 1);
        }
        const tspans = childElementsNamedNS(selectedText, SVG_NS, "tspan");
        for (let tspan of tspans) {
            const option = document.createElement("option");
            option.textContent = tspan.textContent;
            textForm.textSpanSelect.appendChild(option);
        }
        textForm.textSpanSelect.selectedIndex = 0;
        textSpanSelected();

        textForm.editLabelSection.style.display = "";
    }

    export function textDeselected(): void {
        if (textForm === null) {
            return;
        }

        textForm.editLabelSection.style.display = "none";
        selectedText = null;
        selectedTextSpan = null;
    }

    function textSpanSelected(): void {
        if (textForm === null) {
            return;
        }

        // pick out the text-span element
        if (selectedText === null) {
            return;
        }
        const textSpans = <SVGTSpanElement[]>childElementsNamedNS(selectedText, SVG_NS, "tspan");

        const textSpanIndex = textForm.textSpanSelect.selectedIndex;
        if (textSpanIndex === -1) {
            selectedTextSpan = null;
        } else if (textSpanIndex >= textSpans.length) {
            // invalid selection
            selectedTextSpan = null;
        } else {
            selectedTextSpan = textSpans[textSpanIndex];
        }

        if (selectedTextSpan === null) {
            return;
        }

        // update the form

        let leading = 0;
        const lineHeight = pointsValue(selectedTextSpan.style.lineHeight);
        const fontSize = pointsValue(selectedTextSpan.style.fontSize) ?? 12;
        if (lineHeight !== null) {
            leading = lineHeight - fontSize;
        }

        textForm.textArea.value = selectedTextSpan.textContent ?? "";
        textForm.fontBoldCheckbox.checked = selectedTextSpan.style.fontWeight === "bold";
        textForm.fontItalicCheckbox.checked = selectedTextSpan.style.fontStyle === "italic";
        textForm.fontSizeInput.value = "" + (pointsValue(selectedTextSpan.style.fontSize) ?? 12);
        textForm.charSpacingInput.value = "" + (pointsValue(selectedTextSpan.style.letterSpacing) ?? 0);
        textForm.wordSpacingInput.value = "" + (pointsValue(selectedTextSpan.style.wordSpacing) ?? 0);
        textForm.leadingInput.value = "" + leading;
        textForm.languageEnabledCheckbox.checked = selectedTextSpan.hasAttribute("data-lang");
        textForm.languageInput.value = selectedTextSpan.getAttribute("data-lang") ?? "";
        textForm.altTextEnabledCheckbox.checked = selectedTextSpan.hasAttribute("data-alt-text");
        textForm.altTextInput.value = selectedTextSpan.getAttribute("data-alt-text") ?? "";
        textForm.actualTextEnabledCheckbox.checked = selectedTextSpan.hasAttribute("data-actual-text");
        textForm.actualTextInput.value = selectedTextSpan.getAttribute("data-actual-text") ?? "";
        textForm.expansionEnabledCheckbox.checked = selectedTextSpan.hasAttribute("data-expansion");
        textForm.expansionTextInput.value = selectedTextSpan.getAttribute("data-expansion") ?? "";
    }

    function addTextSpan(): void {
        if (textForm === null) {
            return;
        }
        if (selectedText === null) {
            return;
        }

        const newTextChunk = Annotations.createDefaultTextChunk("lorem ipsum");
        Annotations.makeTSpanFromTextChunk(selectedText, newTextChunk);

        // also add an option to the selection box and select it
        const option = document.createElement("option");
        option.text = newTextChunk.text;
        textForm.textSpanSelect.appendChild(option);
        textForm.textSpanSelect.selectedIndex = textForm.textSpanSelect.options.length - 1;
    }

    function removeTextSpan(): void {
        if (textForm === null) {
            return;
        }
        if (selectedText === null) {
            return;
        }
        if (textForm.textSpanSelect.selectedIndex === -1) {
            return;
        }

        const tspans = childElementsNamedNS(selectedText, SVG_NS, "tspan");
        if (textForm.textSpanSelect.selectedIndex >= tspans.length) {
            return;
        }
        const tspanToDelete = tspans[textForm.textSpanSelect.selectedIndex];
        if (tspanToDelete.parentElement !== null) {
            tspanToDelete.parentElement.removeChild(tspanToDelete);
        }

        textForm.textSpanSelect.options.remove(textForm.textSpanSelect.selectedIndex);
    }

    function updateTextSpan(): void {
        if (textForm === null) {
            return;
        }
        if (selectedTextSpan === null) {
            return;
        }

        const lineHeight = (+textForm.leadingInput.value) + (+textForm.fontSizeInput.value);

        selectedTextSpan.textContent = textForm.textArea.value;
        selectedTextSpan.style.fontWeight = textForm.fontBoldCheckbox.checked ? "bold" : "";
        selectedTextSpan.style.fontStyle = textForm.fontItalicCheckbox.checked ? "italic" : "";
        selectedTextSpan.style.fontSize = `${textForm.fontSizeInput.value}pt`;
        selectedTextSpan.style.letterSpacing = `${textForm.charSpacingInput.value}pt`;
        selectedTextSpan.style.wordSpacing = `${textForm.wordSpacingInput.value}pt`;
        selectedTextSpan.style.lineHeight = `${lineHeight}pt`;

        const CHECKBOXES_INPUTS_AND_ATTRIBUTES: [HTMLInputElement, HTMLInputElement|HTMLTextAreaElement, string][] = [
            [textForm.languageEnabledCheckbox, textForm.languageInput, "data-lang"],
            [textForm.altTextEnabledCheckbox, textForm.altTextInput, "data-alt-text"],
            [textForm.actualTextEnabledCheckbox, textForm.actualTextInput, "data-actual-text"],
            [textForm.expansionEnabledCheckbox, textForm.expansionTextInput, "data-expansion"],
        ];
        for (let [checkbox, input, attribute] of CHECKBOXES_INPUTS_AND_ATTRIBUTES) {
            if (checkbox.checked) {
                selectedTextSpan.setAttribute(attribute, input.value);
            } else {
                selectedTextSpan.removeAttribute(attribute);
            }
        }

        // update name of option too
        if (textForm.textSpanSelect.selectedIndex !== -1) {
            textForm.textSpanSelect.options[textForm.textSpanSelect.selectedIndex].textContent = selectedTextSpan.textContent;
        }
    }

    function doInit(): void {
        const editLabelSection = <HTMLDivElement|null>document.getElementById("pdfmcr-edit-label");

        const textSpanSelect = <HTMLSelectElement|null>document.getElementById("pdfmcr-tspan-select");

        const textArea = <HTMLTextAreaElement|null>document.getElementById("pdfmcr-textarea");
        const fontBoldCheckbox = <HTMLInputElement|null>document.getElementById("pdfmcr-font-bold-checkbox");
        const fontItalicCheckbox = <HTMLInputElement|null>document.getElementById("pdfmcr-font-italic-checkbox");
        const fontSizeInput = <HTMLInputElement|null>document.getElementById("pdfmcr-font-size");
        const charSpacingInput = <HTMLInputElement|null>document.getElementById("pdfmcr-char-spacing");
        const wordSpacingInput = <HTMLInputElement|null>document.getElementById("pdfmcr-word-spacing");
        const leadingInput = <HTMLInputElement|null>document.getElementById("pdfmcr-leading");

        const languageEnabledCheckbox = <HTMLInputElement|null>document.getElementById("pdfmcr-lang-enabled");
        const languageInput = <HTMLInputElement|null>document.getElementById("pdfmcr-lang");
        const altTextEnabledCheckbox = <HTMLInputElement|null>document.getElementById("pdfmcr-alt-text-enabled");
        const altTextInput = <HTMLTextAreaElement|null>document.getElementById("pdfmcr-alt-text");
        const actualTextEnabledCheckbox = <HTMLInputElement|null>document.getElementById("pdfmcr-actual-text-enabled");
        const actualTextInput = <HTMLTextAreaElement|null>document.getElementById("pdfmcr-actual-text");
        const expansionEnabledCheckbox = <HTMLInputElement|null>document.getElementById("pdfmcr-expansion-enabled");
        const expansionTextInput = <HTMLTextAreaElement|null>document.getElementById("pdfmcr-expansion");

        if (editLabelSection === null) { return; }
        if (textSpanSelect === null) { return; }
        if (textArea === null) { return; }
        if (fontBoldCheckbox === null) { return; }
        if (fontItalicCheckbox === null) { return; }
        if (fontSizeInput === null) { return; }
        if (charSpacingInput === null) { return; }
        if (wordSpacingInput === null) { return; }
        if (leadingInput === null) { return; }
        if (languageEnabledCheckbox === null) { return; }
        if (languageInput === null) { return; }
        if (altTextEnabledCheckbox === null) { return; }
        if (altTextInput === null) { return; }
        if (actualTextEnabledCheckbox === null) { return; }
        if (actualTextInput === null) { return; }
        if (expansionEnabledCheckbox === null) { return; }
        if (expansionTextInput === null) { return; }

        const addTextSpanButton = <HTMLInputElement|null>document.getElementById("pdfmcr-add-tspan-button");
        const removeTextSpanButton = <HTMLInputElement|null>document.getElementById("pdfmcr-remove-tspan-button");
        const updateTextSpanButton = <HTMLInputElement|null>document.getElementById("pdfmcr-update-tspan-button");

        if (addTextSpanButton === null) { return; }
        if (removeTextSpanButton === null) { return; }
        if (updateTextSpanButton === null) { return; }

        textForm = {
            editLabelSection,
            textSpanSelect,
            textArea,
            fontBoldCheckbox,
            fontItalicCheckbox,
            fontSizeInput,
            charSpacingInput,
            wordSpacingInput,
            leadingInput,
            languageEnabledCheckbox,
            languageInput,
            altTextEnabledCheckbox,
            altTextInput,
            actualTextEnabledCheckbox,
            actualTextInput,
            expansionEnabledCheckbox,
            expansionTextInput,
        };

        textSpanSelect.addEventListener("change", textSpanSelected);
        addTextSpanButton.addEventListener("click", addTextSpan);
        removeTextSpanButton.addEventListener("click", removeTextSpan);
        updateTextSpanButton.addEventListener("click", updateTextSpan);
    }

    export function init(): void {
        document.addEventListener("DOMContentLoaded", doInit);
    }
}
