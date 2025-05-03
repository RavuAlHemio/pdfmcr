import { getImageHeightPt, Position, positionFromTranslate, SVG_NS } from "./common";
import { Annotation, PageAnnotations, TextChunk } from "./model";
import { SvgDrag } from './svgdrag';
import { TextManagement } from "./textmgmt";

export namespace Annotations {
    let dragStart: Position|null = null;
    let dragEvents: ["mousemove"|"mouseup", any][] = [];
    let selectedRect: SVGRectElement|null = null;

    function setGroupPos(annotationGroup: SVGGElement, clientX: number, clientY: number): SVGSVGElement|null {
        if (dragStart === null) {
            return null;
        }

        const svgRoot = annotationGroup.ownerSVGElement;
        if (svgRoot === null) {
            return null;
        }

        const transform = svgRoot.createSVGTransform();
        transform.setTranslate(
            (clientX - dragStart.x) / SvgDrag.currentImageScale,
            (clientY - dragStart.y) / SvgDrag.currentImageScale,
        );
        annotationGroup.transform.baseVal.initialize(transform);

        return svgRoot;
    }

    function rectMoved(annotationGroup: SVGGElement, event: MouseEvent): void {
        setGroupPos(annotationGroup, event.clientX, event.clientY);
    }

    function rectReleased(annotationGroup: SVGGElement, event: MouseEvent): void {
        if (event.button !== 0) {
            // not the left mouse button; ignore
            return;
        }

        const svgRoot = setGroupPos(annotationGroup, event.clientX, event.clientY);
        if (svgRoot === null) {
            return;
        }

        // forget all events
        const removeTheseEvents = dragEvents.splice(0, dragEvents.length);
        for (let removeThisEvent of removeTheseEvents) {
            svgRoot.removeEventListener(removeThisEvent[0], removeThisEvent[1]);
        }

        // forget the drag state
        dragStart = null;
    }

    function rectGrabbed(rect: SVGRectElement, annotationGroup: SVGGElement, event: MouseEvent): void {
        if (event.button !== 0) {
            // not the left mouse button; ignore
            return;
        }

        // do not pass through to group
        event.stopPropagation();

        // color the previous rect white again
        if (selectedRect !== null) {
            selectedRect.style.fill = "#fff";
        }
        selectedRect = rect;

        // color our rect red
        rect.style.fill = "#f00";

        // tell the editor form that things have changed
        const annotationTexts = annotationGroup.getElementsByTagNameNS(SVG_NS, "text");
        if (annotationTexts.length > 0) {
            TextManagement.textSelected(<SVGTextElement>annotationTexts[0]);
        }

        const svgRoot = annotationGroup.ownerSVGElement;
        if (svgRoot === null) {
            return;
        }

        // where is the group now?
        const curPos = positionFromTranslate(annotationGroup, SVGLength.SVG_LENGTHTYPE_NUMBER);
        if (curPos === null) {
            return;
        }

        dragStart = {
            x: event.offsetX - (curPos.x * SvgDrag.currentImageScale),
            y: event.offsetY - (curPos.y * SvgDrag.currentImageScale),
        };

        // set up more events
        const moveEvent = e => rectMoved(annotationGroup, e);
        dragEvents.push(["mousemove", moveEvent]);
        svgRoot.addEventListener("mousemove", moveEvent);

        const releaseEvent = e => rectReleased(annotationGroup, e);
        dragEvents.push(["mouseup", releaseEvent]);
        svgRoot.addEventListener("mouseup", releaseEvent);
    }

    export function createDefaultTextChunk(initialText: string): TextChunk {
        return {
            text: initialText,
            font_variant: "Regular",
            character_spacing: 0,
            word_spacing: 0,
            language: null,
            alternate_text: null,
            actual_text: null,
            expansion: null,
        };
    }

    function createDefaultAnnotation(initialText: string): Annotation {
        return {
            left: 0,
            bottom: 0,
            font_size: 12,
            leading: 0,
            elements: [
                createDefaultTextChunk(initialText),
            ],
        };
    }

    export function makeTSpanFromTextChunk(annoTextElem: SVGTextElement, textChunk: TextChunk): SVGTSpanElement {
        const annoTSpanElem = document.createElementNS(SVG_NS, "tspan");
        annoTSpanElem.style.letterSpacing = `${textChunk.character_spacing}pt`;
        annoTSpanElem.style.wordSpacing = `${textChunk.word_spacing}pt`;
        annoTSpanElem.style.whiteSpace = "pre";
        annoTextElem.appendChild(annoTSpanElem);

        if (textChunk.language !== null) {
            annoTSpanElem.setAttribute("data-language", textChunk.language);
        }
        if (textChunk.alternate_text !== null) {
            annoTSpanElem.setAttribute("data-alternate-text", textChunk.alternate_text);
        }
        if (textChunk.actual_text !== null) {
            annoTSpanElem.setAttribute("data-actual-text", textChunk.actual_text);
        }
        if (textChunk.expansion !== null) {
            annoTSpanElem.setAttribute("data-expansion", textChunk.expansion);
        }

        const annoTextNode = document.createTextNode(textChunk.text);
        annoTSpanElem.appendChild(annoTextNode);

        return annoTSpanElem;
    }

    function makeGroupFromAnnotation(pageGroup: SVGGElement, pageHeightPt: number, annotation: Annotation): SVGGElement|null {
        const svgRoot = pageGroup.ownerSVGElement;
        if (svgRoot === null) {
            return null;
        }

        // convert coordinates to pixels
        const lengther = svgRoot.createSVGLength();

        lengther.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, annotation.left);
        lengther.convertToSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_NUMBER);
        const xPx = lengther.valueInSpecifiedUnits;

        lengther.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, pageHeightPt - annotation.bottom);
        lengther.convertToSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_NUMBER);
        const yPx = lengther.valueInSpecifiedUnits;

        const annoGroup = document.createElementNS(SVG_NS, "g");
        annoGroup.classList.add("annotation");
        const transform = svgRoot.createSVGTransform();
        transform.setTranslate(xPx, yPx);
        annoGroup.transform.baseVal.initialize(transform);

        const lineHeight = annotation.font_size + annotation.leading;

        const annoTextElem = document.createElementNS(SVG_NS, "text");
        annoTextElem.style.fill = "#000";
        annoTextElem.style.fontSize = `${annotation.font_size}pt`;
        annoTextElem.style.lineHeight = `${lineHeight}pt`;
        annoGroup.appendChild(annoTextElem);

        for (let element of annotation.elements) {
            makeTSpanFromTextChunk(annoTextElem, element);
        }

        const grabRect = document.createElementNS(SVG_NS, "rect");
        grabRect.x.baseVal.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, 0);
        grabRect.y.baseVal.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, 0);
        grabRect.width.baseVal.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, 10);
        grabRect.height.baseVal.newValueSpecifiedUnits(SVGLength.SVG_LENGTHTYPE_PT, 10);
        grabRect.style.fill = "#fff";
        grabRect.style.fillOpacity = "0.5";
        grabRect.addEventListener("mousedown", event => rectGrabbed(grabRect, annoGroup, event));
        annoGroup.appendChild(grabRect);

        pageGroup.appendChild(annoGroup);

        return annoGroup;
    }

    function newAnnotationFormSubmit(textBox: HTMLInputElement, event: SubmitEvent): void {
        event.preventDefault();
        const annotationText = textBox.value;

        const pageGroup = <SVGGElement|null>document.getElementById("pdfmcr-page-group");
        if (pageGroup === null) {
            return;
        }

        const imageHeightPt = getImageHeightPt(pageGroup);
        if (imageHeightPt === null) {
            return;
        }

        const freshAnnotation = createDefaultAnnotation(annotationText);
        makeGroupFromAnnotation(pageGroup, imageHeightPt, freshAnnotation);
    }

    function hookUpNewAnnotationForm(): void {
        const newAnnotationForm = <HTMLFormElement|null>document.getElementById("pdfmcr-new-annotation-form");
        if (newAnnotationForm === null) {
            return;
        }

        const textBox = <HTMLInputElement|null>newAnnotationForm.querySelector("input[type=text]");
        if (textBox === null) {
            return;
        }

        newAnnotationForm.addEventListener("submit", event => newAnnotationFormSubmit(textBox, event));
    }

    function realizeExistingAnnotations(existingAnnotations: PageAnnotations): void {
        const pageGroup = <SVGGElement|null>document.getElementById("pdfmcr-page-group");
        if (pageGroup === null) {
            return;
        }

        const imageHeightPt = getImageHeightPt(pageGroup);
        if (imageHeightPt === null) {
            return;
        }

        for (let annotation of existingAnnotations.annotations) {
            makeGroupFromAnnotation(pageGroup, imageHeightPt, annotation);
        }
    }

    function doInit(existingAnnotations: PageAnnotations): void {
        realizeExistingAnnotations(existingAnnotations);
        hookUpNewAnnotationForm();
    }

    export function init(existingAnnotations?: PageAnnotations|undefined): void {
        if (existingAnnotations === undefined) {
            existingAnnotations = {
                annotations: [],
                artifacts: [],
            };
        }
        document.addEventListener("DOMContentLoaded", () => doInit(existingAnnotations));
    }
}
