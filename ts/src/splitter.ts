import { BadMap, NoValue } from "./badmap.js";


interface Position {
    x: number;
    y: number;
}

interface StartState {
    startPosition: Position,
    splitterLeft: number,
    splitterTop: number,
    leftPaneWidth: number,
    rightPaneWidth: number,
}


export namespace Splitter {
    const activeDocumentEvents: ["mousemove"|"mouseup", (event: MouseEvent) => void][] = [];
    const splitterToStartState: BadMap<HTMLElement, StartState> = new BadMap();

    function mouseReleased(): void {
        // disable the active document events
        const disableUs = activeDocumentEvents.splice(0, activeDocumentEvents.length);
        for (let i = 0; i < disableUs.length; i++) {
            document.removeEventListener(disableUs[i][0], disableUs[i][1]);
        }
    }

    function mouseMoved(splitter: HTMLElement, event: MouseEvent): void {
        const startState = splitterToStartState.get(splitter);
        if (startState === NoValue) {
            return;
        }

        const leftPane = <HTMLElement|null>splitter.previousElementSibling;
        const rightPane = <HTMLElement|null>splitter.nextElementSibling;
        if (leftPane === null || rightPane === null) {
            return;
        }

        const difference = {
            x: event.clientX - startState.startPosition.x,
            y: event.clientY - startState.startPosition.y,
        };
        if (difference.x < -startState.leftPaneWidth) {
            difference.x = -startState.leftPaneWidth;
        }
        if (difference.x > startState.rightPaneWidth) {
            difference.x = startState.rightPaneWidth;
        }

        splitter.style.left = `${startState.splitterLeft + difference.x}px`;
        leftPane.style.width = `${startState.leftPaneWidth + difference.x}px`;
        rightPane.style.width = `${startState.rightPaneWidth - difference.x}px`;
    }

    function activateSplitter(splitter: HTMLElement, event: MouseEvent): void {
        const leftPane = <HTMLElement|null>splitter.previousElementSibling;
        const rightPane = <HTMLElement|null>splitter.nextElementSibling;
        if (leftPane === null || rightPane === null) {
            return;
        }

        splitterToStartState.set(splitter, {
            startPosition: {
                x: event.clientX,
                y: event.clientY,
            },
            splitterLeft: splitter.offsetLeft,
            splitterTop: splitter.offsetTop,
            leftPaneWidth: leftPane.offsetWidth,
            rightPaneWidth: rightPane.offsetWidth,
        });

        const moveHandler = moveEvent => mouseMoved(splitter, moveEvent);
        document.addEventListener("mousemove", moveHandler);
        activeDocumentEvents.push(["mousemove", moveHandler]);

        const releaseHandler = () => mouseReleased();
        document.addEventListener("mouseup", releaseHandler);
        activeDocumentEvents.push(["mouseup", releaseHandler]);
    }

    function initContainer(container: HTMLElement): void {
        const panes = <HTMLCollectionOf<HTMLElement>>container.getElementsByClassName("spl-pane");
        for (const pane of panes) {
            pane.style.flexGrow = "1";
            pane.style.flexShrink = "1";
            pane.style.flexBasis = "auto";
            pane.style.overflow = "hidden";
        }
        const splitters = <HTMLCollectionOf<HTMLElement>>container.getElementsByClassName("spl-splitter");
        for (const splitter of splitters) {
            splitter.style.width = "8px";
            splitter.style.height = "100%";
            splitter.style.cursor = "col-resize";
            splitter.style.userSelect = "none";
            splitter.addEventListener("mousedown", event => activateSplitter(splitter, event));
        }
    }

    function doInit(): void {
        // initialize each splitter container
        const splitterContainers = document.getElementsByClassName("spl-splitter-container");
        for (const splitterContainer of splitterContainers) {
            initContainer(<HTMLElement>splitterContainer);
        }
    }

    export function init(): void {
        document.addEventListener("DOMContentLoaded", doInit);
    }
}
