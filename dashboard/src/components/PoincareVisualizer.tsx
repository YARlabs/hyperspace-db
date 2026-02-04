import { useEffect, useRef, useState } from 'react'

interface PoincareVisualizerProps {
    apiKey: string
    collections: string[]
    selectedCollection: string | null
}

interface Vector {
    id: number
    coords: number[]
    metadata?: Record<string, string>
}

export default function PoincareVisualizer({ apiKey: _apiKey, collections: _collections, selectedCollection }: PoincareVisualizerProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null)
    const [vectors, setVectors] = useState<Vector[]>([])
    const [loading, setLoading] = useState(false)

    // Fetch sample vectors from collection
    const fetchVectors = async (_collectionName: string) => {
        setLoading(true)
        try {
            // For now, generate sample data since we don't have a "list vectors" endpoint
            // In production, you'd call an API endpoint
            const sampleVectors: Vector[] = []
            for (let i = 0; i < 50; i++) {
                const angle = (i / 50) * 2 * Math.PI
                const radius = 0.3 + Math.random() * 0.5
                sampleVectors.push({
                    id: i,
                    coords: [
                        radius * Math.cos(angle),
                        radius * Math.sin(angle)
                    ]
                })
            }
            setVectors(sampleVectors)
        } catch (err) {
            console.error(err)
        } finally {
            setLoading(false)
        }
    }

    useEffect(() => {
        if (selectedCollection) {
            fetchVectors(selectedCollection)
        }
    }, [selectedCollection])

    useEffect(() => {
        const canvas = canvasRef.current
        if (!canvas) return

        const ctx = canvas.getContext('2d')
        if (!ctx) return

        const width = canvas.width
        const height = canvas.height
        const centerX = width / 2
        const centerY = height / 2
        const radius = Math.min(width, height) / 2 - 20

        // Clear canvas
        ctx.fillStyle = '#000'
        ctx.fillRect(0, 0, width, height)

        // Draw Poincaré disk boundary
        ctx.strokeStyle = '#00ffff'
        ctx.lineWidth = 2
        ctx.beginPath()
        ctx.arc(centerX, centerY, radius, 0, 2 * Math.PI)
        ctx.stroke()

        // Draw grid lines (hyperbolic geodesics approximation)
        ctx.strokeStyle = '#333'
        ctx.lineWidth = 1
        for (let i = 0; i < 8; i++) {
            const angle = (i / 8) * 2 * Math.PI
            ctx.beginPath()
            ctx.moveTo(centerX, centerY)
            ctx.lineTo(
                centerX + radius * Math.cos(angle),
                centerY + radius * Math.sin(angle)
            )
            ctx.stroke()
        }

        // Draw concentric circles
        for (let r = 0.25; r <= 1; r += 0.25) {
            ctx.beginPath()
            ctx.arc(centerX, centerY, radius * r, 0, 2 * Math.PI)
            ctx.stroke()
        }

        // Draw vectors
        vectors.forEach((vec, idx) => {
            if (vec.coords.length < 2) return

            const x = centerX + vec.coords[0] * radius
            const y = centerY + vec.coords[1] * radius

            // Check if point is inside unit disk
            const dist = Math.sqrt(vec.coords[0] ** 2 + vec.coords[1] ** 2)
            if (dist >= 1) return

            // Draw point
            ctx.fillStyle = `hsl(${(idx * 360) / vectors.length}, 70%, 60%)`
            ctx.beginPath()
            ctx.arc(x, y, 4, 0, 2 * Math.PI)
            ctx.fill()

            // Draw ID label
            ctx.fillStyle = '#fff'
            ctx.font = '10px monospace'
            ctx.fillText(`${vec.id}`, x + 6, y - 6)
        })

        // Draw title
        ctx.fillStyle = '#00ffff'
        ctx.font = 'bold 16px monospace'
        ctx.fillText('Poincaré Disk Model', 10, 25)

        if (selectedCollection) {
            ctx.fillStyle = '#fff'
            ctx.font = '12px monospace'
            ctx.fillText(`Collection: ${selectedCollection}`, 10, 45)
            ctx.fillText(`Vectors: ${vectors.length}`, 10, 60)
        }

    }, [vectors, selectedCollection])

    return (
        <div className="card">
            <h2>Poincaré Disk Visualizer</h2>
            <p style={{ color: '#888', fontSize: '14px' }}>
                Interactive visualization of hyperbolic vector space. Select a collection to view its data distribution.
            </p>

            {!selectedCollection && (
                <div style={{ padding: '40px', textAlign: 'center', color: '#666' }}>
                    Select a collection from the Collections tab to visualize
                </div>
            )}

            {selectedCollection && (
                <>
                    {loading && <p>Loading vectors...</p>}
                    <canvas
                        ref={canvasRef}
                        width={800}
                        height={800}
                        style={{
                            width: '100%',
                            maxWidth: '800px',
                            border: '1px solid #333',
                            borderRadius: '8px',
                            display: 'block',
                            margin: '20px auto'
                        }}
                    />
                    <div style={{ marginTop: '20px', fontSize: '12px', color: '#888' }}>
                        <p>• Each point represents a vector in hyperbolic space</p>
                        <p>• Distance from center indicates magnitude in Poincaré model</p>
                        <p>• Colors differentiate individual vectors</p>
                    </div>
                </>
            )}
        </div>
    )
}
