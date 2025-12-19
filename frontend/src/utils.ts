import { api, apiRequest, fetchAPI, type Analysis, type AnalysesSearchResult } from './api'

export async function lastAnalysisByHash(hash: string | null): Promise<Analysis | null> {
  if (!hash) return null

  const params = new URLSearchParams()
  params.append('hash', hash)

  const search_res = await fetchAPI<AnalysesSearchResult>(
    apiRequest(api.endpoints.analysesSearch, undefined, params),
  )

  if (!search_res) {
    return null
  }

  const analyses = search_res.analyses

  if (!analyses.length) {
    return null
  }

  return analyses[0]
}
