import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { AssetAddCard } from '../../components/asset-add-card'
import * as services from '../../components/services'
import React from 'react'

vi.mock('../../components/services', () => ({
  handleGetAllProjects: vi.fn(),
  handleAddAsset: vi.fn(),
}))

// Mock UI components
vi.mock('../../components/ui/card', () => ({
  Card: ({ children }: any) => <div>{children}</div>,
  CardHeader: ({ children }: any) => <header>{children}</header>,
  CardTitle: ({ children }: any) => <h1>{children}</h1>,
  CardDescription: ({ children }: any) => <p>{children}</p>,
  CardContent: ({ children }: any) => <div>{children}</div>,
  CardFooter: ({ children }: any) => <footer>{children}</footer>,
}))

vi.mock('../../components/ui/select', () => ({
  Select: ({ children, onValueChange }: any) => (
    <div data-testid="select">
      <select onChange={(e) => onValueChange(e.target.value)}>
        <option value="">Select a project</option>
        <option value="1">Proj 1</option>
      </select>
      {children}
    </div>
  ),
  SelectTrigger: ({ children }: any) => <div>{children}</div>,
  SelectValue: ({ placeholder }: any) => <span>{placeholder}</span>,
  SelectContent: ({ children }: any) => <div>{children}</div>,
  SelectItem: ({ value, children }: any) => <option value={value}>{children}</option>,
}))

vi.mock('../../components/ui/input', () => ({
  Input: (props: any) => <input {...props} />,
}))

vi.mock('../../components/ui/label', () => ({
  Label: ({ children, htmlFor }: any) => <label htmlFor={htmlFor}>{children}</label>,
}))

vi.mock('../../components/ui/button', () => ({
  Button: ({ children, onClick }: any) => <button onClick={onClick}>{children}</button>,
}))

describe('AssetAddCard', () => {
  const mockOnClose = vi.fn()

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(services.handleGetAllProjects).mockResolvedValue([{ id: 1, name: 'Proj 1' }] as any)
  })

  it('renders correctly and fetches projects', async () => {
    render(<AssetAddCard onClose={mockOnClose} />)
    expect(screen.getByText('Created asset')).toBeInTheDocument()
    await waitFor(() => {
      expect(services.handleGetAllProjects).toHaveBeenCalled()
    })
  })

  it('calls handleAddAsset on create', async () => {
    render(<AssetAddCard onClose={mockOnClose} />)
    
    fireEvent.change(screen.getByPlaceholderText('Asset name'), { target: { value: 'New Asset' } })
    
    const select = screen.getByRole('combobox') // Select is rendered as a select because of our mock
    fireEvent.change(select, { target: { value: '1' } })
    
    fireEvent.click(screen.getByText('Create Asset'))
    
    expect(services.handleAddAsset).toHaveBeenCalledWith('New Asset', 1)
  })

  it('calls onClose when clicking cancel', () => {
    render(<AssetAddCard onClose={mockOnClose} />)
    fireEvent.click(screen.getByText('Cancel'))
    expect(mockOnClose).toHaveBeenCalled()
  })
})
