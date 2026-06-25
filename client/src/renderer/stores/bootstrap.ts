import { defineStore } from 'pinia'
import { useFilePolicyStore } from './file-policy'
import { useWhitelistStore } from './whitelist'

export const useBootstrapStore = defineStore('bootstrap', () => {
  function clear(): void {
    useFilePolicyStore().clear()
    useWhitelistStore().clear()
  }

  return {
    clear,
  }
})
