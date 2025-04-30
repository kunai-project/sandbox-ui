import { api, apiRequest, fetchAPI, type Analysis } from './api'

export async function lastAnalysisByHash(hash: string | null): Promise<Analysis | null> {
  if (!hash) return null

  const params = new URLSearchParams()
  params.append('hash', hash)

  const search_res = await fetchAPI<[Analysis]>(
    apiRequest(api.endpoints.analysesSearch, undefined, params),
  )

  if (!search_res) {
    return null
  }

  if (!search_res.length) {
    return null
  }

  return search_res[0]
}
