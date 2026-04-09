import { describe, it, expect, vi, beforeEach } from 'vitest'
import { 
  handleGetAllAssets, 
  handleAddAsset, 
  handleDeleteAsset,
  handleGetAllCategories,
  handleAddCategory,
  handleGetAllProjects,
  handleAddProject
} from '../../components/services'
import { invoke } from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('services', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Assets Services', () => {
    it('handleGetAllAssets returns assets on success', async () => {
      const mockAssets = [{ id: 1, name: 'Asset 1' }]
      vi.mocked(invoke).mockResolvedValue(mockAssets)
      
      const result = await handleGetAllAssets()
      expect(result).toEqual(mockAssets)
      expect(invoke).toHaveBeenCalledWith('get_all_assets')
    })

    it('handleGetAllAssets returns empty array on failure', async () => {
      vi.mocked(invoke).mockRejectedValue(new Error('Failed'))
      
      const result = await handleGetAllAssets()
      expect(result).toEqual([])
    })

    it('handleAddAsset calls invoke with correct arguments', async () => {
      await handleAddAsset('New Asset', 1)
      expect(invoke).toHaveBeenCalledWith('add_asset', { name: 'New Asset', projectId: 1 })
    })

    it('handleDeleteAsset calls invoke with correct arguments', async () => {
      await handleDeleteAsset('Asset to delete')
      expect(invoke).toHaveBeenCalledWith('delete_asset', { name: 'Asset to delete' })
    })
  })

  describe('Categories Services', () => {
    it('handleGetAllCategories returns categories', async () => {
      const mockCategories = [{ id: 1, name: 'Cat 1' }]
      vi.mocked(invoke).mockResolvedValue(mockCategories)
      
      const result = await handleGetAllCategories()
      expect(result).toEqual(mockCategories)
    })

    it('handleAddCategory calls invoke', async () => {
      await handleAddCategory('New Cat')
      expect(invoke).toHaveBeenCalledWith('add_category', { name: 'New Cat' })
    })
  })

  describe('Projects Services', () => {
    it('handleGetAllProjects returns projects', async () => {
      const mockProjects = [{ id: 1, name: 'Project 1' }]
      vi.mocked(invoke).mockResolvedValue(mockProjects)
      
      const result = await handleGetAllProjects()
      expect(result).toEqual(mockProjects)
    })

    it('handleAddProject calls invoke', async () => {
      await handleAddProject('New Project')
      expect(invoke).toHaveBeenCalledWith('add_project', { name: 'New Project' })
    })
  })
})
