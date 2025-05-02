import { Position } from "./common";


export namespace SvgDrag {
    let dragStart: Position|null = null;
    let currentImageScale: number = 1.0;
    let currentImageOffset: Position = { x: 0, y: 0 };
    const dragEvents: [string, any][] = [];

    function updateGroupTransform(groupElem: SVGGElement): void {
        const svgRoot = groupElem.ownerSVGElement;
        if (svgRoot === null) {
            // can't do much
            return;
        }
        const scaling = svgRoot.createSVGTransform();
        scaling.setScale(currentImageScale, currentImageScale);
        const translation = svgRoot.createSVGTransform();
        translation.setTranslate(currentImageOffset.x, currentImageOffset.y);
        groupElem.transform.baseVal.initialize(scaling);
        groupElem.transform.baseVal.appendItem(translation);
    }

    function resetView(groupElem: SVGGElement): void {
        currentImageOffset = {
            x: 0,
            y: 0,
        };
        currentImageScale = 1.0;
        updateGroupTransform(groupElem);
    }

    function performZoom(groupElem: SVGGElement, factor: number): void {
        currentImageScale *= factor;
        updateGroupTransform(groupElem);
    }

    function groupDragOver(groupElem: SVGGElement, overEvent: MouseEvent): void {
        if (dragStart === null) {
            // can't do much
            return;
        }

        currentImageOffset = {
            x: (overEvent.clientX - dragStart.x) / currentImageScale,
            y: (overEvent.clientY - dragStart.y) / currentImageScale,
        };

        // move the group there
        updateGroupTransform(groupElem);
    }

    function groupDragEnd(groupElem: SVGGElement, endEvent: MouseEvent): void {
        if (endEvent.button !== 0) {
            // wrong button released
            return;
        }

        if (dragStart === null) {
            // meh
            return;
        }
        const svgRoot = groupElem.ownerSVGElement;
        if (svgRoot === null) {
            return;
        }

        // store new position as final coordinates
        currentImageOffset = {
            x: (endEvent.clientX - dragStart.x) / currentImageScale,
            y: (endEvent.clientY - dragStart.y) / currentImageScale,
        };

        // forget start coordinates
        dragStart = null;

        // unregister all the drag events
        const oldDragEvents = dragEvents.splice(0, dragEvents.length);
        for (let oldDragEvent of oldDragEvents) {
            svgRoot.removeEventListener(oldDragEvent[0], oldDragEvent[1]);
        }

        // update one last time
        updateGroupTransform(groupElem);
    }

    function registerDragEvent<K extends keyof SVGElementEventMap>(element: SVGElement, eventName: K, handler: (this: SVGElement, ev: SVGElementEventMap[K]) => any) {
        element.addEventListener(eventName, handler);
        dragEvents.push([eventName, handler]);
    }

    function groupDragStarted(groupElem: SVGGElement, startEvent: MouseEvent): void {
        if (startEvent.button !== 0) {
            return;
        }
        const svgRoot = groupElem.ownerSVGElement;
        if (svgRoot === null) {
            return;
        }

        dragStart = {
            x: startEvent.offsetX - (currentImageOffset.x * currentImageScale),
            y: startEvent.offsetY - (currentImageOffset.y * currentImageScale),
        };
        registerDragEvent(svgRoot, "mousemove", overEvent => groupDragOver(groupElem, overEvent));
        registerDragEvent(svgRoot, "mouseup", endEvent => groupDragEnd(groupElem, endEvent));
    }

    function doInit(): void {
        const groupElem = <SVGGElement|null>document.getElementById("pdfmcr-page-group");
        if (groupElem === null) {
            return;
        }
        groupElem.addEventListener("mousedown", startEvent => groupDragStarted(groupElem, startEvent));

        const zoomInButton = <HTMLInputElement|null>document.getElementById("pdfmcr-zoom-in-button");
        if (zoomInButton !== null) {
            zoomInButton.addEventListener("click", () => performZoom(groupElem, 3.0/2.0));
        }
        const zoomOutButton = <HTMLInputElement|null>document.getElementById("pdfmcr-zoom-out-button");
        if (zoomOutButton !== null) {
            zoomOutButton.addEventListener("click", () => performZoom(groupElem, 2.0/3.0));
        }

        const resetViewButton = <HTMLInputElement|null>document.getElementById("pdfmcr-reset-view-button");
        if (resetViewButton !== null) {
            resetViewButton.addEventListener("click", () => resetView(groupElem));
        }
    }

    export function init(): void {
        document.addEventListener("DOMContentLoaded", doInit);
    }
}
