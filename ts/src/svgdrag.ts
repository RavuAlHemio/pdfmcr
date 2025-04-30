import { Position } from "./common";


export namespace SvgDrag {
    let dragStart: Position|null = null;
    let currentImageOffset: Position = { x: 0, y: 0 };
    const dragEvents: [string, any][] = [];

    function updateGroupTransform(groupElem: SVGGElement): void {
        const svgRoot = groupElem.ownerSVGElement;
        if (svgRoot === null) {
            // can't do much
            return;
        }
        const transform = svgRoot.createSVGTransform();
        transform.setTranslate(currentImageOffset.x, currentImageOffset.y);
        groupElem.transform.baseVal.initialize(transform);
    }

    function resetView(groupElem: SVGGElement): void {
        currentImageOffset = {
            x: 0,
            y: 0,
        };
        updateGroupTransform(groupElem);
    }

    function groupDragOver(groupElem: SVGGElement, overEvent: MouseEvent): void {
        if (dragStart === null) {
            // can't do much
            return;
        }

        currentImageOffset = {
            x: overEvent.clientX - dragStart.x,
            y: overEvent.clientY - dragStart.y,
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
            x: endEvent.clientX - dragStart.x,
            y: endEvent.clientY - dragStart.y,
        };

        // forget start coordinates
        dragStart = null;

        // unregister all the drag events
        const oldDragEvents = dragEvents.splice(0, dragEvents.length);
        for (let oldDragEvent of oldDragEvents) {
            svgRoot.removeEventListener(oldDragEvent[0], oldDragEvent[1]);
        }
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
            x: startEvent.offsetX - currentImageOffset.x,
            y: startEvent.offsetY - currentImageOffset.y,
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

        const resetViewButton = <HTMLInputElement|null>document.getElementById("pdfmcr-reset-view-button");
        if (resetViewButton !== null) {
            resetViewButton.addEventListener("click", () => resetView(groupElem));
        }
    }

    export function init(): void {
        document.addEventListener("DOMContentLoaded", doInit);
    }
}
