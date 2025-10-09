import React, { useRef, useMemo } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { fragmentShader, vertexShader } from "../shaders/backgroundShader";
import "./ShaderBackground.css";

interface ShaderBackgroundProps {
    speed?: number;
    intensity?: number;
    scale?: number;
    opacity?: number;
    enabled?: boolean;
}

function ShaderPlane({
    speed = 1.0,
    intensity = 1.0,
    scale = 1.0,
    opacity = 1.0,
}: Omit<ShaderBackgroundProps, "enabled">) {
    const meshRef = useRef<THREE.Mesh>(null);
    const startTime = useRef(Date.now());

    const uniforms = useMemo(
        () => ({
            iTime: { value: 0 },
            iResolution: {
                value: new THREE.Vector2(window.innerWidth, window.innerHeight),
            },
            uSpeed: { value: speed },
            uIntensity: { value: intensity },
            uScale: { value: scale },
            uOpacity: { value: opacity },
        }),
        [],
    );

    // Update uniforms when props change
    React.useEffect(() => {
        if (uniforms.uSpeed) uniforms.uSpeed.value = speed;
        if (uniforms.uIntensity) uniforms.uIntensity.value = intensity;
        if (uniforms.uScale) uniforms.uScale.value = scale;
        if (uniforms.uOpacity) uniforms.uOpacity.value = opacity;
    }, [speed, intensity, scale, opacity, uniforms]);

    // Update resolution on window resize
    React.useEffect(() => {
        const handleResize = () => {
            if (uniforms.iResolution) {
                uniforms.iResolution.value.set(
                    window.innerWidth,
                    window.innerHeight,
                );
            }
        };

        window.addEventListener("resize", handleResize);
        return () => window.removeEventListener("resize", handleResize);
    }, [uniforms]);

    // Animation loop
    useFrame(() => {
        if (meshRef.current) {
            const elapsed = (Date.now() - startTime.current) / 1000;
            if (uniforms.iTime) {
                uniforms.iTime.value = elapsed;
            }
        }
    });

    return (
        <mesh ref={meshRef} scale={[2, 2, 1]}>
            <planeGeometry args={[2, 2]} />
            <shaderMaterial
                vertexShader={vertexShader}
                fragmentShader={fragmentShader}
                uniforms={uniforms}
                transparent={true}
                depthWrite={false}
            />
        </mesh>
    );
}

export function ShaderBackground({
    speed = 0.3,
    intensity = 1.0,
    scale = 1.5,
    opacity = 0.4,
    enabled = true,
}: ShaderBackgroundProps) {
    if (!enabled) return null;

    return (
        <div className="shader-background">
            <Canvas
                camera={{ position: [0, 0, 1], fov: 75 }}
                gl={{
                    alpha: true,
                    antialias: false,
                    powerPreference: "high-performance",
                }}
                dpr={Math.min(window.devicePixelRatio, 2)}
                frameloop="always"
            >
                <ShaderPlane
                    speed={speed}
                    intensity={intensity}
                    scale={scale}
                    opacity={opacity}
                />
            </Canvas>
        </div>
    );
}

export default ShaderBackground;
