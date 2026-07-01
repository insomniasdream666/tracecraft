import React, { useRef, useEffect, useCallback, useImperativeHandle, forwardRef } from "react";
import { Stage, Layer, Line } from "react-konva";
import { useUIStore } from "../../stores/uiStore";
import { Point, Rect } from "../../lib/geometry";
import { GRID_SIZE, MIN_ZOOM, MAX_ZOOM } from "../../lib/constants";

export interface KonvaCanvasHandle {
  zoomToBounds: (bounds: Rect) => void;
}

interface KonvaCanvasProps {
  children?: React.ReactNode;
  width: number;
  height: number;
  onWorldMouseDown?: (pos: Point, evt: any) => void;
  onWorldMouseMove?: (pos: Point, evt: any) => void;
  onWorldMouseUp?: (pos: Point, evt: any) => void;
  onWorldDblClick?: (pos: Point, evt: any) => void;
  onWorldDrop?: (pos: Point, componentId: string) => void;
  showGrid?: boolean;
  onStageReady?: (stage: any) => void;
}

export const KonvaCanvas = forwardRef<KonvaCanvasHandle, KonvaCanvasProps>(function KonvaCanvas({
  children,
  width,
  height,
  onWorldMouseDown,
  onWorldMouseMove,
  onWorldMouseUp,
  onWorldDblClick,
  onWorldDrop,
  showGrid = true,
  onStageReady,
}, ref) {
  const stageRef = useRef<any>(null);
  const { zoom, panOffset, setZoom, setPan } = useUIStore();
  const isPanning = useRef(false);
  const lastPointer = useRef({ x: 0, y: 0 });

  const screenToWorld = useCallback(
    (screenPos: Point): Point => {
      const stage = stageRef.current;
      if (!stage) return screenPos;
      const stagePos = stage.getPosition();
      const scale = stage.scaleX();
      return {
        x: (screenPos.x - stagePos.x) / scale,
        y: (screenPos.y - stagePos.y) / scale,
      };
    },
    []
  );

  const zoomToBounds = useCallback((bounds: Rect) => {
    const stage = stageRef.current;
    if (!stage) return;
    const padding = 0.1;
    const bx = bounds.width + bounds.x * padding * 2;
    const by = bounds.height + bounds.y * padding * 2;
    const scaleX = (width * (1 - padding)) / (bounds.width || 1);
    const scaleY = (height * (1 - padding)) / (bounds.height || 1);
    const newScale = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, Math.min(scaleX, scaleY)));
    const cx = bounds.x + bounds.width / 2;
    const cy = bounds.y + bounds.height / 2;
    const newX = width / 2 - cx * newScale;
    const newY = height / 2 - cy * newScale;
    stage.scale({ x: newScale, y: newScale });
    stage.position({ x: newX, y: newY });
    stage.batchDraw();
    setZoom(newScale);
    setPan({ x: newX, y: newY });
  }, [width, height, setZoom, setPan]);

  useImperativeHandle(ref, () => ({ zoomToBounds }), [zoomToBounds]);

  useEffect(() => {
    const { setCanvasApi } = useUIStore.getState();
    setCanvasApi({ zoomToBounds });
    return () => setCanvasApi(null);
  }, [zoomToBounds]);

  const handleWheel = useCallback(
    (e: any) => {
      e.evt.preventDefault();
      const stage = stageRef.current;
      if (!stage) return;

      const oldScale = stage.scaleX();
      const pointer = stage.getPointerPosition();
      const mousePointTo = {
        x: (pointer.x - stage.x()) / oldScale,
        y: (pointer.y - stage.y()) / oldScale,
      };

      const direction = e.evt.deltaY > 0 ? -1 : 1;
      const scaleBy = 1.1;
      const newScale = direction > 0 ? oldScale * scaleBy : oldScale / scaleBy;
      const clampedScale = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, newScale));

      stage.scale({ x: clampedScale, y: clampedScale });
      stage.position({
        x: pointer.x - mousePointTo.x * clampedScale,
        y: pointer.y - mousePointTo.y * clampedScale,
      });
      stage.batchDraw();

      setZoom(clampedScale);
      setPan({ x: stage.x(), y: stage.y() });
    },
    [setZoom, setPan]
  );

  const handleMouseDown = useCallback(
    (e: any) => {
      if (e.evt.button === 1 || e.evt.button === 0) {
        isPanning.current = true;
        const pos = e.target.getStage().getPointerPosition();
        lastPointer.current = { x: pos.x, y: pos.y };
      }
      if (e.evt.button === 0 && onWorldMouseDown) {
        const pos = e.target.getStage().getPointerPosition();
        onWorldMouseDown(screenToWorld({ x: pos.x, y: pos.y }), e);
      }
    },
    [onWorldMouseDown, screenToWorld]
  );

  const handleMouseMove = useCallback(
    (e: any) => {
      const stage = stageRef.current;
      if (!stage) return;

      if (isPanning.current) {
        const pos = stage.getPointerPosition();
        const dx = pos.x - lastPointer.current.x;
        const dy = pos.y - lastPointer.current.y;
        stage.position({ x: stage.x() + dx, y: stage.y() + dy });
        stage.batchDraw();
        lastPointer.current = { x: pos.x, y: pos.y };
        setPan({ x: stage.x(), y: stage.y() });
      }

      if (onWorldMouseMove) {
        const pos = stage.getPointerPosition();
        onWorldMouseMove(screenToWorld({ x: pos.x, y: pos.y }), e);
      }
    },
    [onWorldMouseMove, screenToWorld, setPan]
  );

  const handleMouseUp = useCallback(
    (e: any) => {
      isPanning.current = false;
      if (onWorldMouseUp) {
        const stage = stageRef.current;
        if (stage) {
          const pos = stage.getPointerPosition();
          onWorldMouseUp(screenToWorld({ x: pos.x, y: pos.y }), e);
        }
      }
    },
    [onWorldMouseUp, screenToWorld]
  );

  useEffect(() => {
    if (onStageReady && stageRef.current) {
      onStageReady(stageRef.current);
    }
  }, [onStageReady]);

  useEffect(() => {
    const container = stageRef.current?.container();
    if (!container) return;
    const handleDragOver = (e: DragEvent) => { e.preventDefault(); };
    const handleDrop = (e: DragEvent) => {
      e.preventDefault();
      const rect = container.getBoundingClientRect();
      const worldPos = screenToWorld({ x: e.clientX - rect.left, y: e.clientY - rect.top });
      const componentId = e.dataTransfer?.getData("componentId");
      if (componentId && onWorldDrop) {
        onWorldDrop(worldPos, componentId);
      }
    };
    container.addEventListener("dragover", handleDragOver);
    container.addEventListener("drop", handleDrop);
    return () => {
      container.removeEventListener("dragover", handleDragOver);
      container.removeEventListener("drop", handleDrop);
    };
  }, [onWorldDrop, screenToWorld]);

  const gridLines = [];
  if (showGrid) {
    const gridSize = GRID_SIZE * zoom;
    const startX = -width;
    const startY = -height;
    const endX = width * 2;
    const endY = height * 2;
    for (let x = startX; x < endX; x += gridSize) {
      gridLines.push(
        <Line key={`gv${x}`} points={[x, startY, x, endY]} stroke="#2D2F52" strokeWidth={0.5} opacity={0.3} />
      );
    }
    for (let y = startY; y < endY; y += gridSize) {
      gridLines.push(
        <Line key={`gh${y}`} points={[startX, y, endX, y]} stroke="#2D2F52" strokeWidth={0.5} opacity={0.3} />
      );
    }
  }

  return (
    <Stage
      ref={stageRef}
      width={width}
      height={height}
      onWheel={handleWheel}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
      onDblClick={(e: any) => {
        if (onWorldDblClick) {
          const stage = stageRef.current;
          if (stage) {
            const pos = stage.getPointerPosition();
            onWorldDblClick(screenToWorld({ x: pos.x, y: pos.y }), e);
          }
        }
      }}
      draggable={false}
      scaleX={zoom}
      scaleY={zoom}
      x={panOffset.x}
      y={panOffset.y}
      className="bg-bg-canvas"
    >
      {showGrid && <Layer listening={false}>{gridLines}</Layer>}
      {children}
    </Stage>
  );
});
