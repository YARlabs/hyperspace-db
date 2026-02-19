import { useState } from "react"
import { useAuth } from "@/hooks/use-auth"
import { useNavigate, useLocation } from "react-router-dom"
import { Card, CardHeader, CardTitle, CardContent, CardDescription, CardFooter } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { setAuthToken, api } from "@/lib/api"

export function AuthPage() {
    const [key, setKey] = useState("")
    const [error, setError] = useState("")
    const [loading, setLoading] = useState(false)
    const { setApiKey } = useAuth()
    const navigate = useNavigate()
    const location = useLocation()

    // @ts-ignore
    const from = location.state?.from?.pathname || "/"

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()
        setLoading(true)
        setError("")

        // Validate key against server
        setAuthToken(key)
        try {
            await api.get("/status")
            setApiKey(key)
            navigate(from, { replace: true })
        } catch (err) {
            setError("Invalid API Key or Server Unreachable")
            setAuthToken("") // reset
        } finally {
            setLoading(false)
        }
    }

    return (
        <div className="flex items-center justify-center min-h-screen bg-background p-4">
            <div
                className="absolute inset-0 bg-center [mask-image:linear-gradient(180deg,white,rgba(255,255,255,0))]"
                style={{
                    backgroundImage: `linear-gradient(to right, hsl(var(--primary) / 0.1) 1px, transparent 1px), linear-gradient(to bottom, hsl(var(--primary) / 0.1) 1px, transparent 1px)`,
                    backgroundSize: '40px 40px'
                }}
            ></div>
            <Card className="w-full max-w-md relative z-10 border-primary/20 shadow-[0_0_30px_rgba(124,58,237,0.1)]">
                <CardHeader>
                    <div className="flex justify-center mb-4">
                        <div className="h-16 w-16 flex items-center justify-center font-bold text-primary text-3xl">
                            [H]
                        </div>
                    </div>
                    <CardTitle className="text-2xl text-center">HyperspaceDB Console</CardTitle>
                    <CardDescription className="text-center">Secure Control Plane Access</CardDescription>
                </CardHeader>
                <form onSubmit={handleSubmit}>
                    <CardContent className="space-y-4">
                        <div className="space-y-2">
                            <Label htmlFor="key">API Key</Label>
                            <Input
                                id="key"
                                type="password"
                                placeholder="hs_..."
                                value={key}
                                onChange={e => setKey(e.target.value)}
                                className="font-mono"
                            />
                        </div>
                        {error && <div className="text-xs text-destructive text-center font-medium">{error}</div>}
                    </CardContent>
                    <CardFooter>
                        <Button type="submit" className="w-full" disabled={loading}>
                            {loading ? "Verifying..." : "Connect"}
                        </Button>
                    </CardFooter>
                </form>
            </Card>
        </div>
    )
}
