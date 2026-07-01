import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { PropertyPanel } from '../PropertyPanel';

describe('PropertyPanel', () => {
  it('renders title', () => {
    render(<PropertyPanel title="Properties" properties={[]} />);
    expect(screen.getByText('Properties')).toBeInTheDocument();
  });

  it('renders text inputs', () => {
    render(
      <PropertyPanel
        title="Test"
        properties={[{ label: 'Name', value: 'R1', onChange: vi.fn(), type: 'text' }]}
      />
    );
    const input = screen.getByDisplayValue('R1');
    expect(input).toBeInTheDocument();
    expect(input).toHaveAttribute('type', 'text');
  });

  it('renders number inputs', () => {
    render(
      <PropertyPanel
        title="Test"
        properties={[{ label: 'Value', value: 1000, onChange: vi.fn(), type: 'number' }]}
      />
    );
    const input = screen.getByDisplayValue('1000');
    expect(input).toBeInTheDocument();
    expect(input).toHaveAttribute('type', 'number');
  });

  it('renders dropdown options for select type', () => {
    render(
      <PropertyPanel
        title="Test"
        properties={[{
          label: 'Layer', value: 'top', onChange: vi.fn(), type: 'select',
          options: [
            { label: 'Top', value: 'top' },
            { label: 'Bottom', value: 'bottom' },
          ],
        }]}
      />
    );
    const select = screen.getByRole('combobox');
    expect(select).toBeInTheDocument();
    expect(screen.getByText('Top')).toBeInTheDocument();
    expect(screen.getByText('Bottom')).toBeInTheDocument();
  });

  it('renders boolean as checkbox', () => {
    render(
      <PropertyPanel
        title="Test"
        properties={[{ label: 'Locked', value: true, onChange: vi.fn(), type: 'boolean' }]}
      />
    );
    const checkbox = screen.getByRole('checkbox');
    expect(checkbox).toBeInTheDocument();
    expect(checkbox).toBeChecked();
  });

  it('renders readonly value when no onChange', () => {
    render(
      <PropertyPanel
        title="Test"
        properties={[{ label: 'RefDes', value: 'R1' }]}
      />
    );
    expect(screen.getByText('R1')).toBeInTheDocument();
  });

  it('calls onChange when text input changes', async () => {
    const handleChange = vi.fn();
    render(
      <PropertyPanel
        title="Test"
        properties={[{ label: 'Name', value: 'R1', onChange: handleChange, type: 'text' }]}
      />
    );
    const input = screen.getByDisplayValue('R1');
    await userEvent.clear(input);
    await userEvent.type(input, 'R2');
    expect(handleChange).toHaveBeenCalled();
  });

  it('calls onChange when select changes', async () => {
    const handleChange = vi.fn();
    render(
      <PropertyPanel
        title="Test"
        properties={[{
          label: 'Layer', value: 'top', onChange: handleChange, type: 'select',
          options: [{ label: 'Top', value: 'top' }, { label: 'Bottom', value: 'bottom' }],
        }]}
      />
    );
    const select = screen.getByRole('combobox');
    await userEvent.selectOptions(select, 'bottom');
    expect(handleChange).toHaveBeenCalledWith('bottom');
  });

  it('calls onChange when number changes', async () => {
    const handleChange = vi.fn();
    render(
      <PropertyPanel
        title="Test"
        properties={[{ label: 'Width', value: 0.25, onChange: handleChange, type: 'number' }]}
      />
    );
    const input = screen.getByDisplayValue('0.25');
    await userEvent.clear(input);
    await userEvent.type(input, '0.5');
    expect(handleChange).toHaveBeenCalled();
  });
});
