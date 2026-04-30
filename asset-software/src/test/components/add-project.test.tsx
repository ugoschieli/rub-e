import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { vi, describe, it, expect } from 'vitest'
import AddProjet from '../../components/add-project'
import * as services from '../../components/services'
import { useData } from '../../context/data-context'

vi.mock('../../context/data-context', () => ({
  useData: vi.fn(() => ({
    refreshData: vi.fn().mockResolvedValue(undefined),
  })),
}))

vi.mock('../../components/services', () => ({
  handleAddProject: vi.fn().mockResolvedValue(undefined),
}))

describe('AddProjet', () => {
  it('renders correctly', () => {
    render(<AddProjet />)
    expect(screen.getByPlaceholderText('New project')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: '+' })).toBeInTheDocument()
  })

  it('calls handleAddProject on submit and clears input', async () => {
    render(<AddProjet />)
    const input = screen.getByPlaceholderText('New project')
    const button = screen.getByRole('button', { name: '+' })

    fireEvent.change(input, { target: { value: 'My New Project' } })
    fireEvent.click(button)

    expect(services.handleAddProject).toHaveBeenCalledWith('My New Project')
    
    await waitFor(() => {
      expect(input).toHaveValue('')
    })
  })
})
