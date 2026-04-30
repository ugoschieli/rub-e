import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { vi, describe, it, expect } from 'vitest'
import AddCategory from '../../components/add-categorie'
import * as services from '../../components/services'
import { useData } from '../../context/data-context'

vi.mock('../../context/data-context', () => ({
  useData: vi.fn(() => ({
    refreshData: vi.fn().mockResolvedValue(undefined),
  })),
}))

vi.mock('../../components/services', () => ({
  handleAddCategory: vi.fn().mockResolvedValue(undefined),
}))

describe('AddCategory', () => {
  it('renders correctly', () => {
    render(<AddCategory />)
    expect(screen.getByPlaceholderText('New category')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '+' })).toBeInTheDocument()
  })

  it('calls handleAddCategory on submit and clears input', async () => {
    render(<AddCategory />)
    const input = screen.getByPlaceholderText('New category')
    const button = screen.getByRole('button', { name: '+' })

    fireEvent.change(input, { target: { value: '3D Models' } })
    fireEvent.click(button)

    expect(services.handleAddCategory).toHaveBeenCalledWith('3D Models')
    
    await waitFor(() => {
      expect(input).toHaveValue('')
    })
  })
})
