export async function lastAnalysisByHash(hash: string | null): Promise<string | null> {
  if (!hash) return null

  const params = new URLSearchParams()
  params.append('hash', hash)

  const search_res = await fetchAPI<[string]>(`/api/analyses/search?${params.toString()}`, {
    method: 'GET',
  })

  if (!search_res) {
    return null
  }

  if (!search_res.length) {
    return null
  }

  return search_res[0]
}

function logApiError(message: string, input: RequestInfo | URL, init?: RequestInit) {
  if (init) {
    console.error(`request=${input} init=${init} error=${message}`)
  } else {
    console.error(`request=${input} error=${message}`)
  }
}

export async function fetchAPI<T>(input: RequestInfo | URL, init?: RequestInit): Promise<T | null> {
  try {
    const response = await fetch(input, init)
    if (response.ok) {
      if (response.headers.get('content-type') != 'application/json') {
        logApiError("api endpoint didn't return json content", input, init)
        return null
      }

      const json = await response.json()
      if (json['error']) {
        logApiError(`api endpoint error: ${json['error']}`, input, init)
        return null
      }
      if (json['data']) {
        const data: T = json['data']
        return data
      }
    } else {
      logApiError(`unexpected status from search API: ${response.status}`, input, init)
    }
  } catch (error) {
    logApiError(`caught exception while querying API: ${error}`, input, init)
  }

  return null
}
