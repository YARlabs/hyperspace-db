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
    const res = await api.get("/status")
    return res.data
}

export const fetchHealth = async () => {
    const res = await api.get("/health")
    return res.data
}

export const fetchCollections = async () => {
    const res = await api.get("/collections")
    return res.data
}

export const searchCollection = async (name: string, payload: any) => {
    const res = await api.post(`/collections/${name}/search`, payload)
    return res.data
}

export const searchBatch = async (name: string, payload: any) => {
    const res = await api.post(`/collections/${name}/search/batch`, payload)
    return res.data
}

export const searchMulti = async (payload: any) => {
    const res = await api.post(`/search/multi`, payload)
    return res.data
}

export const scrollCollection = async (name: string, payload: any) => {
    const res = await api.post(`/collections/${name}/scroll`, payload)
    return res.data
}

export const countFiltered = async (name: string, payload: any) => {
    const res = await api.post(`/collections/${name}/count`, payload)
    return res.data
}

export const updatePayload = async (name: string, id: number, metadata: any) => {
    const res = await api.post(`/collections/${name}/payload`, { id, metadata })
    return res.data
}

export const getCollectionStats = (name: string) => api.get(`/collections/${name}/stats`).then(r => r.data)

export const updateCollectionConfig = (name: string, config: any) => api.patch(`/collections/${name}/config`, config)

export const getPoints = async (name: string, ids: number[]) => {
    const res = await api.get(`/collections/${name}/points`, { params: { ids: ids.join(',') } })
    return res.data
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

