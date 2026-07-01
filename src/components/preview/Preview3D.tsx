import React, { useState, useMemo, useEffect } from "react";
import { Canvas } from "@react-three/fiber";
import { OrbitControls, Grid, PerspectiveCamera, Line as DreiLine, Text } from "@react-three/drei";
import { usePCBStore } from "../../stores/pcbStore";
import { invoke } from "@tauri-apps/api/core";
import { Model3D } from "../../types/library";
import { ModelLoader } from "./ModelLoader";
import * as THREE from "three";

function BoardSubstrate() {
  const { board } = usePCBStore();
  const points = useMemo(() => {
    if (!board?.outline?.shape?.length) return null;
    const contour = board.outline.shape[0];
    if (contour.length < 3) return null;
    const shape = new THREE.Shape();
    shape.moveTo(contour[0].x, contour[0].y);
    for (let i = 1; i < contour.length; i++) {
      shape.lineTo(contour[i].x, contour[i].y);
    }
    shape.closePath();
    return shape;
  }, [board]);

  if (points) {
    return (
      <mesh position={[0, -0.5, 0]} rotation={[-Math.PI / 2, 0, 0]}>
        <shapeGeometry args={[points]} />
        <meshStandardMaterial color="#1B5E20" roughness={0.8} metalness={0.1} side={THREE.DoubleSide} />
      </mesh>
    );
  }

  return (
    <mesh position={[0, -0.5, 0]} rotation={[-Math.PI / 2, 0, 0]}>
      <planeGeometry args={[100, 100]} />
      <meshStandardMaterial color="#1B5E20" roughness={0.8} metalness={0.1} />
    </mesh>
  );
}

function Traces3D({ visible }: { visible: boolean }) {
  const { board } = usePCBStore();
  if (!board || !visible) return null;

  const traces = Object.values(board.traces);
  return (
    <>
      {traces.map((trace, i) => {
        if (trace.points.length < 2) return null;
        const path = new THREE.CatmullRomCurve3(
          trace.points.map((p) => new THREE.Vector3(p.x, 0.01, p.y))
        );
        const geo = new THREE.TubeGeometry(path, trace.points.length * 2, trace.width, 4, false);
        return (
          <mesh key={`trace-${i}`} geometry={geo}>
            <meshStandardMaterial color="#D47A2F" metalness={0.7} roughness={0.3} />
          </mesh>
        );
      })}
    </>
  );
}

function Footprints3D() {
  const { board } = usePCBStore();
  const [modelMap, setModelMap] = useState<Record<string, Model3D | null>>({});
  const footprints = board ? Object.values(board.footprints) : [];

  useEffect(() => {
    if (!board) return;
    const ids = [...new Set(footprints.map((fp) => fp.component_id))];
    if (ids.length === 0) return;
    let cancelled = false;
    invoke<Record<string, Model3D | null>>("get_components_model_3d", { ids })
      .then((data) => { if (!cancelled) setModelMap(data); })
      .catch(() => {});
    return () => { cancelled = true; };
  }, [board?.footprints]);

  if (!board) return null;

  return (
    <>
      {footprints.map((fp, i) => {
        const model = modelMap[fp.component_id] ?? null;
        const fpAngle = (fp.rotation ?? 0) * Math.PI / 180;
        const pos: [number, number, number] = [fp.position.x, 0, fp.position.y];

        if (model) {
          const modelAngle = (model.rotation ?? 0) * Math.PI / 180;
          const totalAngle = fpAngle + modelAngle;
          const ox = model.offset?.x ?? 0;
          const oy = model.offset?.y ?? 0;
          const rotatedOx = ox * Math.cos(fpAngle) - oy * Math.sin(fpAngle);
          const rotatedOy = ox * Math.sin(fpAngle) + oy * Math.cos(fpAngle);
          const modelPos: [number, number, number] = [
            fp.position.x + rotatedOx,
            0,
            fp.position.y + rotatedOy,
          ];
          return (
            <ModelLoader
              key={`fp-model-${i}`}
              path={model.path}
              position={modelPos}
              rotation={[0, -totalAngle, 0]}
            />
          );
        }

        return (
          <group key={`fp-${i}`} position={pos} rotation={[0, -fpAngle, 0]}>
            <mesh>
              <boxGeometry args={[8, 2, 6]} />
              <meshStandardMaterial color="#1a1a1a" roughness={0.5} />
            </mesh>
          </group>
        );
      })}
    </>
  );
}

export const Preview3D = React.memo(function Preview3D() {
  const [showCopper, setShowCopper] = useState(true);
  const [showSilkscreen, setShowSilkscreen] = useState(true);
  const [measuring, setMeasuring] = useState(false);
  const [measurePoints, setMeasurePoints] = useState<THREE.Vector3[]>([]);

  return (
    <div className="w-full h-full bg-bg-canvas relative">
      <div className="absolute top-2 left-2 z-10 flex flex-col gap-1 bg-bg-surface/80 p-2 rounded">
        <label className="flex items-center gap-2 text-xs text-text">
          <input type="checkbox" checked={showCopper} onChange={(e) => setShowCopper(e.target.checked)} />
          Copper
        </label>
        <label className="flex items-center gap-2 text-xs text-text">
          <input type="checkbox" checked={showSilkscreen} onChange={(e) => setShowSilkscreen(e.target.checked)} />
          Silkscreen
        </label>
        <button
          onClick={() => setMeasuring(!measuring)}
          className={`px-2 py-1 text-xs rounded ${measuring ? 'bg-accent text-white' : 'bg-bg-surface border border-border text-text'}`}
        >
          {measuring ? "Measuring..." : "Measure"}
        </button>
      </div>
      <Canvas shadows onPointerDown={(e: any) => {
        if (measuring && e.point) {
          const pt = e.point as THREE.Vector3;
          setMeasurePoints((prev) => {
            const next = [...prev, pt.clone()];
            if (next.length > 2) {
              setMeasuring(false);
              return next.slice(-2);
            }
            return next;
          });
        }
      }}>
        <PerspectiveCamera makeDefault position={[80, 60, 80]} fov={45} />
        <OrbitControls enableDamping dampingFactor={0.1} minDistance={10} maxDistance={300} />
        <ambientLight intensity={0.5} />
        <directionalLight position={[50, 80, 50]} intensity={1} castShadow />
        <directionalLight position={[-50, 30, -50]} intensity={0.3} />
        <Grid
          position={[0, -0.5, 0]}
          args={[200, 200]}
          cellSize={2.54}
          cellThickness={0.5}
          cellColor="#2D2F52"
          sectionSize={25.4}
          sectionThickness={1}
          sectionColor="#5C5E7E"
          fadeDistance={200}
        />
        <BoardSubstrate />
        <Traces3D visible={showCopper} />
        <Footprints3D />
        {measurePoints.length >= 2 && (
          <>
            <DreiLine points={measurePoints} color="#FF4444" lineWidth={2} />
            <Text
              position={[
                (measurePoints[0].x + measurePoints[1].x) / 2,
                (measurePoints[0].y + measurePoints[1].y) / 2 + 5,
                (measurePoints[0].z + measurePoints[1].z) / 2,
              ]}
              fontSize={2}
              color="#FF4444"
            >
              {measurePoints[0].distanceTo(measurePoints[1]).toFixed(1)}mm
            </Text>
          </>
        )}
      </Canvas>
    </div>
  );
});
