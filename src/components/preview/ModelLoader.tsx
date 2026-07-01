import React, { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import * as THREE from "three";

interface Model3DData {
  vertices: number[];
  normals: number[];
}

interface ModelLoaderProps {
  path: string;
  position: [number, number, number];
  rotation?: [number, number, number];
  scale?: number;
}

const geometryCache = new Map<string, THREE.BufferGeometry>();

export function ModelLoader({ path, position, rotation = [0, 0, 0], scale = 1 }: ModelLoaderProps) {
  const [geometry, setGeometry] = useState<THREE.BufferGeometry | null>(null);
  const [error, setError] = useState(false);
  const ref = useRef<THREE.Mesh>(null);
  const prevPathRef = useRef(path);

  useEffect(() => {
    if (prevPathRef.current !== path) {
      setError(false);
      setGeometry(null);
      prevPathRef.current = path;
    }
  }, [path]);

  useEffect(() => {
    if (geometryCache.has(path)) {
      setGeometry(geometryCache.get(path)!);
      return;
    }
    let cancelled = false;
    invoke<Model3DData>("get_3d_model_data", { path })
      .then((data) => {
        if (cancelled) return;
        const geo = new THREE.BufferGeometry();
        geo.setAttribute("position", new THREE.Float32BufferAttribute(data.vertices, 3));
        if (data.normals.length > 0) {
          geo.setAttribute("normal", new THREE.Float32BufferAttribute(data.normals, 3));
        } else {
          geo.computeVertexNormals();
        }
        geometryCache.set(path, geo);
        setGeometry(geo);
      })
      .catch(() => {
        if (!cancelled) setError(true);
      });
    return () => { cancelled = true; };
  }, [path]);

  if (error) {
    return (
      <mesh position={position} rotation={rotation as any} scale={scale}>
        <boxGeometry args={[4, 2, 4]} />
        <meshStandardMaterial color="#E6454A" wireframe />
      </mesh>
    );
  }

  if (!geometry) {
    return (
      <mesh position={position} rotation={rotation as any} scale={scale}>
        <boxGeometry args={[6, 1.5, 4]} />
        <meshStandardMaterial color="#1a1a1a" roughness={0.6} />
      </mesh>
    );
  }

  return (
    <mesh ref={ref} position={position} rotation={rotation as any} scale={scale} geometry={geometry}>
      <meshStandardMaterial color="#888888" roughness={0.5} metalness={0.3} side={THREE.DoubleSide} />
    </mesh>
  );
}
