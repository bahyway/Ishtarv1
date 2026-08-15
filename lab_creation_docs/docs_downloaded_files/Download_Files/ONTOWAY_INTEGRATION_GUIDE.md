# OntoWay KGEditor Integration Guide
## Tribal 3D Visualization Integration Strategy

## 📋 TABLE OF CONTENTS
1. [Architecture Overview](#architecture)
2. [React + WebGPU Integration](#react-webgpu)
3. [Avalonia Desktop Integration](#avalonia)
4. [RustUI Integration](#rustui)
5. [Data Bridge Layer](#data-bridge)
6. [API Endpoints](#api-endpoints)
7. [Stakeholder Customization](#customization)

---

## 1. ARCHITECTURE OVERVIEW {#architecture}

```
┌─────────────────────────────────────────────────────────┐
│                 OntoWay KGEditor                        │
│  ┌───────────────────────────────────────────────────┐  │
│  │          Tribal 3D Visualization Engine           │  │
│  │  ┌──────────────┐  ┌──────────────┐              │  │
│  │  │   Three.js   │  │   WebGPU     │              │  │
│  │  │   (React)    │  │   (Native)   │              │  │
│  │  └──────┬───────┘  └──────┬───────┘              │  │
│  │         │                  │                       │  │
│  │         └────────┬─────────┘                       │  │
│  │                  ▼                                  │  │
│  │        ┌─────────────────┐                         │  │
│  │        │  Data Bridge    │                         │  │
│  │        │  (WebSocket)    │                         │  │
│  │        └────────┬────────┘                         │  │
│  └─────────────────┼──────────────────────────────────┘  │
│                    ▼                                     │
│         ┌──────────────────────┐                        │
│         │  BDBWay PostgreSQL   │                        │
│         │  + Tribal Registry   │                        │
│         └──────────────────────┘                        │
└─────────────────────────────────────────────────────────┘
```

---

## 2. REACT + WEBGPU INTEGRATION {#react-webgpu}

### 2.1 Component Structure

```typescript
// File: src/components/TribalVisualization/index.tsx

import React, { useEffect, useRef, useState } from 'react';
import * as THREE from 'three';
import { TribalAPI } from '../../services/api';

interface TribalNode {
    id: string;
    name: string;
    nameAr: string;
    color: number;
    size: number;
    satellites: number;
    ethnicity: 'ARAB' | 'KURDISH' | 'TURKMEN' | 'UNKNOWN';
    position: { x: number; y: number; z: number };
    population?: number;
    dataCount: number; // Cemetery records, etc.
}

interface TribalVisualizationProps {
    dataSource: string; // 'cemetery' | 'custom'
    stakeholderId: string;
    onNodeClick?: (node: TribalNode) => void;
    onNodeHover?: (node: TribalNode | null) => void;
    filterEthnicity?: string[];
    showSubTribes?: boolean;
}

export const TribalVisualization: React.FC<TribalVisualizationProps> = ({
    dataSource = 'cemetery',
    stakeholderId,
    onNodeClick,
    onNodeHover,
    filterEthnicity = [],
    showSubTribes = true
}) => {
    const containerRef = useRef<HTMLDivElement>(null);
    const sceneRef = useRef<THREE.Scene | null>(null);
    const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
    const [tribalData, setTribalData] = useState<TribalNode[]>([]);
    const [selectedNode, setSelectedNode] = useState<TribalNode | null>(null);
    const [loading, setLoading] = useState(true);

    // Fetch tribal data from API
    useEffect(() => {
        const fetchData = async () => {
            try {
                setLoading(true);
                const data = await TribalAPI.getTribalNetwork({
                    dataSource,
                    stakeholderId,
                    ethnicity: filterEthnicity
                });
                setTribalData(data);
            } catch (error) {
                console.error('Failed to load tribal data:', error);
            } finally {
                setLoading(false);
            }
        };
        
        fetchData();
    }, [dataSource, stakeholderId, filterEthnicity]);

    // Initialize Three.js scene
    useEffect(() => {
        if (!containerRef.current || tribalData.length === 0) return;

        // Scene setup
        const scene = new THREE.Scene();
        scene.fog = new THREE.Fog(0x0a0e1a, 10, 100);
        sceneRef.current = scene;

        // Camera
        const camera = new THREE.PerspectiveCamera(
            75,
            containerRef.current.clientWidth / containerRef.current.clientHeight,
            0.1,
            1000
        );
        camera.position.z = 30;
        camera.position.y = 10;

        // Renderer
        const renderer = new THREE.WebGLRenderer({ 
            antialias: true, 
            alpha: true 
        });
        renderer.setSize(
            containerRef.current.clientWidth,
            containerRef.current.clientHeight
        );
        renderer.setClearColor(0x0a0e1a, 1);
        containerRef.current.appendChild(renderer.domElement);
        rendererRef.current = renderer;

        // Lighting
        const ambientLight = new THREE.AmbientLight(0x404040, 2);
        scene.add(ambientLight);

        const pointLight = new THREE.PointLight(0xFFD700, 2, 100);
        pointLight.position.set(0, 20, 0);
        scene.add(pointLight);

        // Create tribal systems
        tribalData.forEach(tribe => {
            const system = createTribalSystem(tribe);
            scene.add(system);
        });

        // Animation loop
        const animate = () => {
            requestAnimationFrame(animate);
            
            // Rotate camera
            camera.position.x = Math.sin(Date.now() * 0.0001) * 30;
            camera.position.z = Math.cos(Date.now() * 0.0001) * 30;
            camera.lookAt(scene.position);
            
            renderer.render(scene, camera);
        };
        animate();

        // Cleanup
        return () => {
            renderer.dispose();
            containerRef.current?.removeChild(renderer.domElement);
        };
    }, [tribalData]);

    const createTribalSystem = (tribe: TribalNode) => {
        const group = new THREE.Group();

        // Central sphere
        const geometry = new THREE.SphereGeometry(tribe.size, 32, 32);
        const material = new THREE.MeshPhongMaterial({
            color: tribe.color,
            emissive: tribe.color,
            emissiveIntensity: 0.3,
            transparent: true,
            opacity: 0.9
        });
        const sphere = new THREE.Mesh(geometry, material);
        sphere.userData = { tribe };
        group.add(sphere);

        // Add satellites if showSubTribes
        if (showSubTribes) {
            for (let i = 0; i < tribe.satellites; i++) {
                const angle = (i / tribe.satellites) * Math.PI * 2;
                const radius = tribe.size * 3;
                
                const satGeometry = new THREE.SphereGeometry(0.3, 16, 16);
                const satMaterial = new THREE.MeshPhongMaterial({
                    color: tribe.color,
                    emissive: tribe.color,
                    emissiveIntensity: 0.5
                });
                const satellite = new THREE.Mesh(satGeometry, satMaterial);
                
                satellite.position.x = Math.cos(angle) * radius;
                satellite.position.z = Math.sin(angle) * radius;
                satellite.position.y = (Math.random() - 0.5) * 2;
                
                group.add(satellite);
            }
        }

        group.position.set(tribe.position.x, tribe.position.y, tribe.position.z);
        return group;
    };

    if (loading) {
        return <div className="loading">Loading tribal network...</div>;
    }

    return (
        <div className="tribal-visualization-container">
            <div ref={containerRef} className="canvas-container" />
            
            {/* Info Panel */}
            {selectedNode && (
                <div className="info-panel">
                    <h3>{selectedNode.nameAr}</h3>
                    <p>Name: {selectedNode.name}</p>
                    <p>Ethnicity: {selectedNode.ethnicity}</p>
                    <p>Color Code: #{selectedNode.color}</p>
                    <p>Records: {selectedNode.dataCount.toLocaleString()}</p>
                </div>
            )}
        </div>
    );
};
```

### 2.2 WebGPU Enhancement (Optional)

```typescript
// File: src/components/TribalVisualization/webgpu.ts

export class WebGPUTribalRenderer {
    private device: GPUDevice;
    private context: GPUCanvasContext;
    private pipeline: GPURenderPipeline;

    async initialize(canvas: HTMLCanvasElement) {
        // Check WebGPU support
        if (!navigator.gpu) {
            throw new Error('WebGPU not supported');
        }

        const adapter = await navigator.gpu.requestAdapter();
        this.device = await adapter!.requestDevice();
        
        this.context = canvas.getContext('webgpu')!;
        const format = navigator.gpu.getPreferredCanvasFormat();
        
        this.context.configure({
            device: this.device,
            format: format,
            alphaMode: 'premultiplied'
        });

        // Create render pipeline for tribal spheres
        this.createPipeline(format);
    }

    private createPipeline(format: GPUTextureFormat) {
        const shaderModule = this.device.createShaderModule({
            code: `
                @vertex
                fn vs_main(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
                    return vec4<f32>(position, 1.0);
                }
                
                @fragment
                fn fs_main() -> @location(0) vec4<f32> {
                    return vec4<f32>(1.0, 0.84, 0.0, 1.0); // Gold color
                }
            `
        });

        this.pipeline = this.device.createRenderPipeline({
            layout: 'auto',
            vertex: {
                module: shaderModule,
                entryPoint: 'vs_main'
            },
            fragment: {
                module: shaderModule,
                entryPoint: 'fs_main',
                targets: [{ format }]
            },
            primitive: {
                topology: 'triangle-list'
            }
        });
    }

    render(tribalNodes: TribalNode[]) {
        const commandEncoder = this.device.createCommandEncoder();
        const textureView = this.context.getCurrentTexture().createView();

        const renderPass = commandEncoder.beginRenderPass({
            colorAttachments: [{
                view: textureView,
                clearValue: { r: 0.04, g: 0.05, b: 0.1, a: 1.0 },
                loadOp: 'clear',
                storeOp: 'store'
            }]
        });

        renderPass.setPipeline(this.pipeline);
        // ... render tribal spheres
        renderPass.end();

        this.device.queue.submit([commandEncoder.finish()]);
    }
}
```

---

## 3. AVALONIA DESKTOP INTEGRATION {#avalonia}

### 3.1 Avalonia XAML View

```xml
<!-- File: Views/TribalVisualizationView.axaml -->
<UserControl xmlns="https://github.com/avaloniaui"
             xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml"
             xmlns:d="http://schemas.microsoft.com/expression/blend/2008"
             xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
             mc:Ignorable="d" d:DesignWidth="800" d:DesignHeight="450"
             x:Class="OntoWay.Views.TribalVisualizationView">
    
    <Grid>
        <Grid.RowDefinitions>
            <RowDefinition Height="Auto"/>
            <RowDefinition Height="*"/>
            <RowDefinition Height="Auto"/>
        </Grid.RowDefinitions>
        
        <!-- Header -->
        <StackPanel Grid.Row="0" Orientation="Horizontal" Margin="10">
            <TextBlock Text="Tribal Network Visualization" 
                       FontSize="24" FontWeight="Bold"/>
            <ComboBox x:Name="DataSourceCombo" 
                      SelectedIndex="0" Margin="20,0,0,0">
                <ComboBoxItem>Cemetery Data</ComboBoxItem>
                <ComboBoxItem>Custom Data</ComboBoxItem>
                <ComboBoxItem>All Data</ComboBoxItem>
            </ComboBox>
        </StackPanel>
        
        <!-- 3D Canvas using SkiaSharp or similar -->
        <Border Grid.Row="1" Background="#0A0E1A" 
                BorderBrush="#667EEA" BorderThickness="2">
            <Panel x:Name="CanvasPanel"/>
        </Border>
        
        <!-- Info Panel -->
        <Grid Grid.Row="2" Background="#1A1E2A" Padding="15">
            <Grid.ColumnDefinitions>
                <ColumnDefinition Width="*"/>
                <ColumnDefinition Width="*"/>
                <ColumnDefinition Width="*"/>
            </Grid.ColumnDefinitions>
            
            <StackPanel Grid.Column="0">
                <TextBlock Text="Total Tribes:" FontWeight="Bold"/>
                <TextBlock x:Name="TotalTribesText" Text="45" 
                           FontSize="24" Foreground="#FFD700"/>
            </StackPanel>
            
            <StackPanel Grid.Column="1">
                <TextBlock Text="Records:" FontWeight="Bold"/>
                <TextBlock x:Name="TotalRecordsText" Text="340,000" 
                           FontSize="24" Foreground="#90EE90"/>
            </StackPanel>
            
            <StackPanel Grid.Column="2">
                <TextBlock Text="Selected:" FontWeight="Bold"/>
                <TextBlock x:Name="SelectedTribeText" Text="-" 
                           FontSize="24" Foreground="#87CEEB"/>
            </StackPanel>
        </Grid>
    </Grid>
</UserControl>
```

### 3.2 Avalonia Code-Behind with SkiaSharp

```csharp
// File: Views/TribalVisualizationView.axaml.cs

using Avalonia.Controls;
using Avalonia.Threading;
using SkiaSharp;
using System;
using System.Collections.Generic;
using System.Numerics;

namespace OntoWay.Views
{
    public partial class TribalVisualizationView : UserControl
    {
        private List<TribalNode> _tribalNodes;
        private SKCanvas _canvas;
        private DispatcherTimer _animationTimer;
        private float _rotation = 0;

        public TribalVisualizationView()
        {
            InitializeComponent();
            InitializeVisualization();
        }

        private void InitializeVisualization()
        {
            // Load tribal data from BDBWay API
            LoadTribalData();

            // Setup animation timer
            _animationTimer = new DispatcherTimer
            {
                Interval = TimeSpan.FromMilliseconds(16) // ~60 FPS
            };
            _animationTimer.Tick += OnAnimationTick;
            _animationTimer.Start();
        }

        private async void LoadTribalData()
        {
            // Call your BDBWay API
            var apiClient = new BDBWayApiClient();
            _tribalNodes = await apiClient.GetTribalNetworkAsync(
                dataSource: "cemetery",
                stakeholderId: CurrentUser.Id
            );

            TotalTribesText.Text = _tribalNodes.Count.ToString();
            TotalRecordsText.Text = _tribalNodes
                .Sum(t => t.DataCount)
                .ToString("N0");
        }

        private void OnAnimationTick(object sender, EventArgs e)
        {
            _rotation += 0.01f;
            // Invalidate and redraw
            CanvasPanel.InvalidateVisual();
        }

        private void OnPaintSurface(SKCanvas canvas, int width, int height)
        {
            canvas.Clear(SKColors.Transparent);

            var centerX = width / 2f;
            var centerY = height / 2f;

            foreach (var node in _tribalNodes)
            {
                // Calculate 3D to 2D projection
                var projected = Project3DTo2D(
                    node.Position,
                    _rotation,
                    centerX,
                    centerY
                );

                // Draw tribal sphere
                var paint = new SKPaint
                {
                    Color = SKColor.Parse(node.ColorHex),
                    IsAntialias = true,
                    Style = SKPaintStyle.Fill
                };

                canvas.DrawCircle(
                    projected.X,
                    projected.Y,
                    node.Size * 10,
                    paint
                );

                // Draw glow effect
                var glowPaint = new SKPaint
                {
                    Color = SKColor.Parse(node.ColorHex).WithAlpha(50),
                    IsAntialias = true,
                    MaskFilter = SKMaskFilter.CreateBlur(
                        SKBlurStyle.Normal,
                        10
                    )
                };

                canvas.DrawCircle(
                    projected.X,
                    projected.Y,
                    node.Size * 15,
                    glowPaint
                );
            }
        }

        private Vector2 Project3DTo2D(
            Vector3 point3D,
            float rotation,
            float centerX,
            float centerY
        )
        {
            // Simple perspective projection
            var distance = 500f;
            var rotated = Vector3.Transform(
                point3D,
                Matrix4x4.CreateRotationY(rotation)
            );

            var scale = distance / (distance + rotated.Z);
            return new Vector2(
                centerX + rotated.X * scale,
                centerY + rotated.Y * scale
            );
        }
    }

    public class TribalNode
    {
        public string Id { get; set; }
        public string Name { get; set; }
        public string NameAr { get; set; }
        public string ColorHex { get; set; }
        public float Size { get; set; }
        public Vector3 Position { get; set; }
        public int DataCount { get; set; }
        public string Ethnicity { get; set; }
    }
}
```

---

## 4. RUSTUI INTEGRATION {#rustui}

### 4.1 Rust + wgpu (WebGPU for Desktop)

```rust
// File: src/ui/tribal_visualization.rs

use wgpu;
use winit::window::Window;
use cgmath::{Vector3, Matrix4, Deg, perspective};

pub struct TribalNode {
    pub id: String,
    pub name: String,
    pub name_ar: String,
    pub color: [f32; 3],
    pub size: f32,
    pub position: Vector3<f32>,
    pub data_count: u32,
    pub ethnicity: String,
}

pub struct TribalVisualization {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    tribal_nodes: Vec<TribalNode>,
    camera_rotation: f32,
}

impl TribalVisualization {
    pub async fn new(window: &Window) -> Self {
        // Initialize wgpu
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(window) }.unwrap();
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_capabilities(&adapter).formats[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        // Create render pipeline
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tribal Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/tribal.wgsl").into()),
        });

        let render_pipeline = create_render_pipeline(&device, &config, &shader);

        // Load tribal data from BDBWay PostgreSQL
        let tribal_nodes = Self::load_tribal_data().await;

        Self {
            device,
            queue,
            surface,
            config,
            render_pipeline,
            tribal_nodes,
            camera_rotation: 0.0,
        }
    }

    async fn load_tribal_data() -> Vec<TribalNode> {
        // Connect to PostgreSQL
        let client = tokio_postgres::connect(
            "host=/home/akkad/.pgrx port=28816 dbname=bdbway_extension",
            tokio_postgres::NoTls,
        )
        .await
        .unwrap()
        .0;

        let rows = client
            .query(
                "SELECT 
                    tribe_id,
                    tribe_name_ar,
                    tribe_name_en,
                    assigned_color,
                    estimated_population,
                    ethnicity::TEXT
                FROM bdb_tribal_hierarchy
                WHERE tier = 1",
                &[],
            )
            .await
            .unwrap();

        rows.iter()
            .enumerate()
            .map(|(i, row)| {
                let color_value: i32 = row.get(3);
                let color = color_from_value(color_value);
                
                TribalNode {
                    id: row.get::<_, i32>(0).to_string(),
                    name: row.get(2),
                    name_ar: row.get(1),
                    color,
                    size: 2.0,
                    position: calculate_position(i, rows.len()),
                    data_count: row.get::<_, Option<i32>>(4).unwrap_or(0) as u32,
                    ethnicity: row.get(5),
                }
            })
            .collect()
    }

    pub fn update(&mut self, dt: f32) {
        self.camera_rotation += dt * 0.1;
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.04,
                            g: 0.05,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            
            // Render each tribal sphere
            for node in &self.tribal_nodes {
                // Set uniforms for this node
                // Draw sphere geometry
                render_pass.draw(0..36, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn color_from_value(value: i32) -> [f32; 3] {
    match value {
        0..=10 => [0.82, 0.41, 0.12],   // Brown (Arab major)
        31..=50 => [0.56, 0.93, 0.56],  // Green (Kurdish)
        51..=70 => [0.53, 0.81, 0.92],  // Blue (Turkmen)
        _ => [0.75, 0.75, 0.75],        // Gray (other)
    }
}

fn calculate_position(index: usize, total: usize) -> Vector3<f32> {
    let angle = (index as f32 / total as f32) * std::f32::consts::TAU;
    let radius = 15.0;
    Vector3::new(
        angle.cos() * radius,
        (index as f32 - total as f32 / 2.0) * 2.0,
        angle.sin() * radius,
    )
}

fn create_render_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    shader: &wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
}
```

---

## 5. DATA BRIDGE LAYER {#data-bridge}

### 5.1 WebSocket Real-Time Updates

```typescript
// File: src/services/tribalWebSocket.ts

export class TribalWebSocket {
    private ws: WebSocket;
    private reconnectAttempts = 0;
    private maxReconnectAttempts = 5;

    constructor(
        private url: string,
        private onUpdate: (data: TribalNode[]) => void
    ) {
        this.connect();
    }

    private connect() {
        this.ws = new WebSocket(this.url);

        this.ws.onopen = () => {
            console.log('Connected to BDBWay tribal stream');
            this.reconnectAttempts = 0;
            
            // Subscribe to tribal updates
            this.ws.send(JSON.stringify({
                action: 'subscribe',
                channel: 'tribal_network',
                filters: {
                    dataSource: 'cemetery',
                    ethnicity: ['ARAB', 'KURDISH', 'TURKMEN']
                }
            }));
        };

        this.ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            
            if (data.type === 'tribal_update') {
                this.onUpdate(data.nodes);
            }
        };

        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };

        this.ws.onclose = () => {
            console.log('Disconnected from tribal stream');
            this.attemptReconnect();
        };
    }

    private attemptReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            setTimeout(() => {
                console.log(`Reconnecting... (${this.reconnectAttempts}/${this.maxReconnectAttempts})`);
                this.connect();
            }, 2000 * this.reconnectAttempts);
        }
    }

    send(data: any) {
        if (this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(data));
        }
    }

    close() {
        this.ws.close();
    }
}
```

### 5.2 REST API Client

```typescript
// File: src/services/api.ts

export class TribalAPI {
    private static baseURL = 'http://localhost:5000/api';

    static async getTribalNetwork(params: {
        dataSource: string;
        stakeholderId: string;
        ethnicity?: string[];
    }): Promise<TribalNode[]> {
        const query = new URLSearchParams({
            data_source: params.dataSource,
            stakeholder_id: params.stakeholderId,
            ...(params.ethnicity && { 
                ethnicity: params.ethnicity.join(',') 
            })
        });

        const response = await fetch(
            `${this.baseURL}/tribal/network?${query}`
        );

        if (!response.ok) {
            throw new Error(`API error: ${response.statusText}`);
        }

        return response.json();
    }

    static async getTribalStatistics(
        dataSource: string
    ): Promise<TribalStatistics> {
        const response = await fetch(
            `${this.baseURL}/tribal/stats?data_source=${dataSource}`
        );
        return response.json();
    }

    static async updateTribalData(
        nodeId: string,
        updates: Partial<TribalNode>
    ): Promise<void> {
        await fetch(`${this.baseURL}/tribal/node/${nodeId}`, {
            method: 'PATCH',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(updates)
        });
    }
}
```

---

## 6. API ENDPOINTS {#api-endpoints}

### 6.1 Flask Backend

```python
# File: api/tribal_endpoints.py

from flask import Flask, jsonify, request
from flask_cors import CORS
import psycopg2
import psycopg2.extras

app = Flask(__name__)
CORS(app)

DB_CONFIG = {
    'host': '/home/akkad/.pgrx',
    'port': 28816,
    'database': 'bdbway_extension',
    'user': 'akkad'
}

@app.route('/api/tribal/network', methods=['GET'])
def get_tribal_network():
    """Get tribal network for visualization"""
    data_source = request.args.get('data_source', 'cemetery')
    stakeholder_id = request.args.get('stakeholder_id')
    ethnicity_filter = request.args.get('ethnicity', '').split(',')
    
    conn = psycopg2.connect(**DB_CONFIG)
    cursor = conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor)
    
    # Build query
    ethnicity_clause = ""
    if ethnicity_filter and ethnicity_filter[0]:
        placeholders = ','.join(['%s'] * len(ethnicity_filter))
        ethnicity_clause = f"AND h.ethnicity::TEXT IN ({placeholders})"
    
    query = f"""
        WITH tribal_stats AS (
            SELECT 
                data->>'tribal_affiliation' as tribe_name,
                COUNT(*) as data_count
            FROM spatial.fabric_spatial_quads
            WHERE data->>'tribal_affiliation' IS NOT NULL
            GROUP BY data->>'tribal_affiliation'
        )
        SELECT 
            h.tribe_id,
            h.tribe_name_ar,
            h.tribe_name_en,
            h.assigned_color,
            h.tier,
            h.ethnicity::TEXT,
            h.estimated_population,
            COALESCE(ts.data_count, 0) as data_count,
            (SELECT COUNT(*) FROM bdb_tribal_hierarchy 
             WHERE parent_tribe_id = h.tribe_id) as sub_tribe_count
        FROM bdb_tribal_hierarchy h
        LEFT JOIN tribal_stats ts ON ts.tribe_name = h.tribe_name_ar
        WHERE h.tier = 1
        {ethnicity_clause}
        ORDER BY h.estimated_population DESC NULLS LAST
    """
    
    params = ethnicity_filter if ethnicity_clause else []
    cursor.execute(query, params)
    
    tribes = cursor.fetchall()
    conn.close()
    
    # Transform to visualization format
    result = []
    for i, tribe in enumerate(tribes):
        # Calculate 3D position based on ethnicity and index
        position = calculate_3d_position(
            i, 
            len(tribes),
            tribe['ethnicity']
        )
        
        result.append({
            'id': str(tribe['tribe_id']),
            'name': tribe['tribe_name_en'],
            'nameAr': tribe['tribe_name_ar'],
            'color': color_to_hex(tribe['assigned_color']),
            'size': calculate_size(tribe['estimated_population']),
            'satellites': tribe['sub_tribe_count'],
            'ethnicity': tribe['ethnicity'],
            'position': position,
            'population': tribe['estimated_population'],
            'dataCount': tribe['data_count']
        })
    
    return jsonify(result)

def calculate_3d_position(index, total, ethnicity):
    """Calculate 3D position based on ethnicity grouping"""
    import math
    
    # Group by ethnicity in 3D space
    ethnicity_offsets = {
        'ARAB': {'y': 10},
        'KURDISH': {'y': 0},
        'TURKMEN': {'y': -10},
        'UNKNOWN': {'y': -20}
    }
    
    base_offset = ethnicity_offsets.get(ethnicity, {'y': 0})
    
    angle = (index / total) * 2 * math.pi
    radius = 15
    
    return {
        'x': math.cos(angle) * radius,
        'y': base_offset['y'] + (index % 3 - 1) * 3,
        'z': math.sin(angle) * radius
    }

def color_to_hex(color_value):
    """Convert color value to hex"""
    colors = {
        range(0, 11): '#D2691E',    # Brown (Arab major)
        range(11, 31): '#A0522D',   # Sienna (Arab medium)
        range(31, 51): '#90EE90',   # Light green (Kurdish)
        range(51, 71): '#87CEEB',   # Sky blue (Turkmen)
        range(201, 256): '#C0C0C0'  # Silver (non-tribal)
    }
    
    for range_val, hex_color in colors.items():
        if color_value in range_val:
            return hex_color
    
    return '#808080'  # Default gray

def calculate_size(population):
    """Calculate sphere size based on population"""
    if not population:
        return 1.5
    
    # Logarithmic scale for better visualization
    import math
    return 1 + math.log10(population) / 2

@app.route('/api/tribal/stats', methods=['GET'])
def get_tribal_stats():
    """Get tribal statistics"""
    conn = psycopg2.connect(**DB_CONFIG)
    cursor = conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor)
    
    cursor.execute("""
        SELECT * FROM bdb_tribal_statistics
        ORDER BY total_population DESC
    """)
    
    stats = cursor.fetchall()
    conn.close()
    
    return jsonify(stats)

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000, debug=True)
```

---

## 7. STAKEHOLDER CUSTOMIZATION {#customization}

### 7.1 Custom Data Source Integration

```typescript
// File: src/services/stakeholderConfig.ts

export interface StakeholderConfig {
    id: string;
    name: string;
    dataSource: {
        type: 'cemetery' | 'custom' | 'mixed';
        tableName?: string;
        filters?: Record<string, any>;
    };
    visualization: {
        showEthnicity: string[];
        colorScheme: 'default' | 'custom';
        customColors?: Record<string, string>;
        showSubTribes: boolean;
        cameraSpeed: number;
    };
    permissions: {
        canEdit: boolean;
        canExport: boolean;
        canShare: boolean;
    };
}

export class StakeholderConfigManager {
    async loadConfig(stakeholderId: string): Promise<StakeholderConfig> {
        const response = await fetch(
            `/api/stakeholder/${stakeholderId}/config`
        );
        return response.json();
    }

    async saveConfig(
        stakeholderId: string, 
        config: StakeholderConfig
    ): Promise<void> {
        await fetch(`/api/stakeholder/${stakeholderId}/config`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(config)
        });
    }
}
```

### 7.2 Usage Example

```typescript
// In your main app

const stakeholderConfig = await configManager.loadConfig(currentUser.id);

<TribalVisualization
    dataSource={stakeholderConfig.dataSource.type}
    stakeholderId={currentUser.id}
    filterEthnicity={stakeholderConfig.visualization.showEthnicity}
    showSubTribes={stakeholderConfig.visualization.showSubTribes}
    onNodeClick={(node) => {
        console.log('Clicked node:', node);
        // Show detailed view
    }}
/>
```

---

## 8. DEPLOYMENT CHECKLIST

### Web (React + WebGPU)
- [ ] Bundle with Webpack/Vite
- [ ] Deploy to static hosting (Vercel/Netlify)
- [ ] Configure CORS for API
- [ ] Enable WebSocket connection

### Desktop (Avalonia)
- [ ] Build for Windows/Linux/Mac
- [ ] Package with installer
- [ ] Include native dependencies
- [ ] Configure database connection

### Desktop (Rust)
- [ ] Compile with `cargo build --release`
- [ ] Bundle with assets
- [ ] Create platform-specific packages
- [ ] Test GPU compatibility

---

## 9. NEXT STEPS

1. **Choose your primary platform** (Web/Desktop)
2. **Set up the API backend** (Flask + PostgreSQL)
3. **Integrate visualization component**
4. **Add stakeholder authentication**
5. **Implement data filtering**
6. **Add export/sharing features**

Would you like me to create a specific implementation for any of these platforms?
