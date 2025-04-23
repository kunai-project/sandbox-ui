interface Config {
  kunaiLink: string
  sandboxUILink: string
  api: {
    upload: string
    analyzeAgain: string
    listAnalysis: string
    sandboxesList: string
  }
}

export const config: Config = {
  kunaiLink: 'https://github.com/kunai-project/kunai',
  sandboxUILink: 'https://github.com/kunai-project/sandbox-ui',
  api: {
    upload: '/api/analyze',
    analyzeAgain: '/api/analyze/again',
    listAnalysis: '/api/analyses/search',
    sandboxesList: '/api/sandboxes/list',
  },
}
