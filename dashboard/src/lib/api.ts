import axios from "axios"

export const api = axios.create({
    baseURL: "/api",
})

export const setAuthToken = (token: string) => {
    api.defaults.headers.common["x-api-key"] = token
}

api.interceptors.response.use(
    (response) => response,
    (error) => {
        if (error.response?.status === 401) {
            // Only clear if we're not on login page already
            if (localStorage.getItem("hyperspace_api_key") && !window.location.pathname.includes("/login")) {
                localStorage.removeItem("hyperspace_api_key")
                window.location.href = "/login"
            }
        }
        return Promise.reject(error)
    }
)

const token = localStorage.getItem("hyperspace_api_key")
if (token) setAuthToken(token)

export const fetchStatus = async () => {
    try {
        const res = await api.get("/status")
        return res.data
    } catch {
        const res = await api.get("/cluster/status")
        return res.data
    }
}

export const fetchTrajectoryHistory = async () => {
    const res = await api.get("/admin/trajectory/history")
    return res.data
}

export const startMigrationService = async () => {
    const res = await api.post("/admin/migration/start")
    return res.data
}

export const getMigrationEngineStatus = async () => {
    const res = await api.get("/admin/migration/status")
    return res.data
}

export const startMigrationTask = async (config: any) => {
    const res = await api.post("/admin/migration/task", config)
    return res.data
}

export const getMigrationTaskStatus = async (taskId: string) => {
    const res = await api.get(`/admin/migration/task/${taskId}`)
    return res.data
}

