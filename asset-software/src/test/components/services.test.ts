import { describe, it, expect, vi, beforeEach } from 'vitest'
import { 
  handleGetAllAssets, 
  handleAddAsset, 
  handleDeleteAsset,
  handleAddCategoryToAsset,
  handleAddProjectToAsset,
  handleUpdateAssetCategoryAndProject,
  handleGetAllCategories,
  handleAddCategory,
  handleDeleteCategory,
  handleGetAllProjects,
  handleAddProject,
  handleDeleteProject
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

    it('handleAddCategoryToAsset calls invoke and returns result', async () => {
      vi.mocked(invoke).mockResolvedValue('success')
      const result = await handleAddCategoryToAsset('Asset 1', { id: 10, name: 'Cat 10' })
      expect(invoke).toHaveBeenCalledWith('add_category_to_asset', {
        assetName: 'Asset 1',
        category: { id: 10, name: 'Cat 10' }
      })
      expect(result).toBe('success')
    })

    it('handleAddProjectToAsset calls invoke and returns result', async () => {
      vi.mocked(invoke).mockResolvedValue('success')
      const result = await handleAddProjectToAsset('Asset 1', { id: 20, name: 'Proj 20' })
      expect(invoke).toHaveBeenCalledWith('add_project_to_asset', {
        assetName: 'Asset 1',
        project: { id: 20, name: 'Proj 20' }
      })
      expect(result).toBe('success')
    })

    it('handleAddAsset handles error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      vi.mocked(invoke).mockRejectedValue(new Error('Failed'))
      await handleAddAsset('New Asset', 1)
      expect(consoleSpy).toHaveBeenCalledWith('Failed to add asset:', expect.any(Error))
      consoleSpy.mockRestore()
    })

    it('handleDeleteAsset handles error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      vi.mocked(invoke).mockRejectedValue(new Error('Failed'))
      await handleDeleteAsset('Asset to delete')
      expect(consoleSpy).toHaveBeenCalledWith('Failed to delete asset:', expect.any(Error))
      consoleSpy.mockRestore()
    })

    it('handleAddCategoryToAsset handles error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      vi.mocked(invoke).mockRejectedValue(new Error('Failed'))
      await expect(handleAddCategoryToAsset('Asset 1', { id: 10, name: 'Cat 10' })).rejects.toThrow()
      expect(consoleSpy).toHaveBeenCalledWith('Failed to add category to asset:', expect.any(Error))
      consoleSpy.mockRestore()
    })

    it('handleAddProjectToAsset handles error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      vi.mocked(invoke).mockRejectedValue(new Error('Failed'))
      await expect(handleAddProjectToAsset('Asset 1', { id: 20, name: 'Proj 20' })).rejects.toThrow()
      expect(consoleSpy).toHaveBeenCalledWith('Failed to add project to asset:', expect.any(Error))
      consoleSpy.mockRestore()
    })

    it('handleUpdateAssetCategoryAndProject handles error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      vi.mocked(invoke).mockRejectedValue(new Error('Failed'))
      await handleUpdateAssetCategoryAndProject(1, 10, 20)
      expect(consoleSpy).toHaveBeenCalledWith('Failed to update asset category and project:', expect.any(Error))
      consoleSpy.mockRestore()
    })
  })

  describe('Categories Services', () => {
    it('handleGetAllCategories handles error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      vi.mocked(invoke).mockRejectedValue(new Error('Failed'))
      const result = await handleGetAllCategories()
      expect(result).toBeUndefined()
      expect(consoleSpy).toHaveBeenCalledWith('Failed to get all categories:', expect.any(Error))
      consoleSpy.mockRestore()
    })

    it('handleAddCategory handles error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      vi.mocked(invoke).mockRejectedValue(new Error('Failed'))
      await handleAddCategory('New Cat')
      expect(consoleSpy).toHaveBeenCalledWith('Failed to add category:', expect.any(Error))
      consoleSpy.mockRestore()
    })

    it('handleDeleteCategory handles error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      vi.mocked(invoke).mockRejectedValue(new Error('Failed'))
      await handleDeleteCategory('Cat to delete')
      expect(consoleSpy).toHaveBeenCalledWith('Failed to delete category:', expect.any(Error))
      consoleSpy.mockRestore()
    })
  })

  describe('Projects Services', () => {
    it('handleGetAllProjects handles error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      vi.mocked(invoke).mockRejectedValue(new Error('Failed'))
      const result = await handleGetAllProjects()
      expect(result).toEqual([])
      expect(consoleSpy).toHaveBeenCalledWith('Failed to get all projects:', expect.any(Error))
      consoleSpy.mockRestore()
    })

    it('handleAddProject handles error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      vi.mocked(invoke).mockRejectedValue(new Error('Failed'))
      await handleAddProject('New Project')
      expect(consoleSpy).toHaveBeenCalledWith('Failed to add project:', expect.any(Error))
      consoleSpy.mockRestore()
    })

    it('handleDeleteProject handles error', async () => {
      const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})
      vi.mocked(invoke).mockRejectedValue(new Error('Failed'))
      await handleDeleteProject('Project to delete')
      expect(consoleSpy).toHaveBeenCalledWith('Failed to delete project:', expect.any(Error))
      consoleSpy.mockRestore()
    })
  })
})
