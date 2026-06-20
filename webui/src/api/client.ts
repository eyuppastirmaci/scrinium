const BASE_URL = '/api'

export async function get<T>(path: string): Promise<T> {
  const response = await fetch(`${BASE_URL}${path}`, {
    headers: { 'Accept': 'application/json' },
  })

  if (!response.ok) {
    const body = await response.text()
    let message: string
    try {
      const parsed = JSON.parse(body)
      message = parsed.message ?? parsed.error ?? body
    } catch {
      message = body || response.statusText
    }
    throw new Error(message)
  }

  return response.json()
}
