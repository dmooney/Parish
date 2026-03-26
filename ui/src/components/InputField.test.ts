import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, fireEvent } from '@testing-library/svelte';
import { streamingActive, npcsHere } from '../stores/game';
import InputField from './InputField.svelte';

// Mock ipc submitInput
vi.mock('$lib/ipc', () => ({
	submitInput: vi.fn(async (_text: string) => {})
}));

describe('InputField', () => {
	beforeEach(() => {
		streamingActive.set(false);
		npcsHere.set([]);
	});

	it('renders an input field', () => {
		const { getByRole } = render(InputField);
		expect(getByRole('textbox')).toBeTruthy();
	});

	it('has correct placeholder when idle', () => {
		const { getByPlaceholderText } = render(InputField);
		expect(getByPlaceholderText('What do you do? (@ to mention NPC)')).toBeTruthy();
	});

	it('is disabled when streaming', () => {
		streamingActive.set(true);
		const { getByRole } = render(InputField);
		expect((getByRole('textbox') as HTMLInputElement).disabled).toBe(true);
	});

	it('clears input after submit', async () => {
		const { getByRole } = render(InputField);
		const input = getByRole('textbox') as HTMLInputElement;
		await fireEvent.input(input, { target: { value: 'hello' } });
		await fireEvent.keyDown(input, { key: 'Enter' });
		// Input should be cleared
		expect(input.value).toBe('');
	});

	describe('NPC mention autocomplete', () => {
		const testNpcs = [
			{ name: 'Padraig Darcy', occupation: 'Publican', mood: 'content', introduced: true },
			{ name: 'Siobhan Murphy', occupation: 'Farmer', mood: 'determined', introduced: true },
			{ name: 'Father Callahan', occupation: 'Priest', mood: 'serene', introduced: false }
		];

		beforeEach(() => {
			npcsHere.set(testNpcs);
		});

		it('shows mention dropdown when @ is typed with a letter', async () => {
			const { getByRole, queryByRole } = render(InputField);
			const input = getByRole('textbox') as HTMLInputElement;

			// No dropdown initially
			expect(queryByRole('listbox')).toBeNull();

			// Type @P
			await fireEvent.input(input, { target: { value: '@P' } });
			expect(queryByRole('listbox')).toBeTruthy();
		});

		it('filters NPCs by typed text', async () => {
			const { getByRole, queryAllByRole } = render(InputField);
			const input = getByRole('textbox') as HTMLInputElement;

			// Type @P — should show Padraig only
			await fireEvent.input(input, { target: { value: '@P' } });
			const options = queryAllByRole('option');
			expect(options.length).toBe(1);
			expect(options[0].textContent).toContain('Padraig Darcy');
		});

		it('shows all NPCs when only @ and first letter matches multiple', async () => {
			const { getByRole, queryAllByRole } = render(InputField);
			const input = getByRole('textbox') as HTMLInputElement;

			// Type @S — only Siobhan starts with S
			await fireEvent.input(input, { target: { value: '@S' } });
			const options = queryAllByRole('option');
			expect(options.length).toBe(1);
			expect(options[0].textContent).toContain('Siobhan Murphy');
		});

		it('does not show dropdown when no NPCs present', async () => {
			npcsHere.set([]);
			const { getByRole, queryByRole } = render(InputField);
			const input = getByRole('textbox') as HTMLInputElement;

			await fireEvent.input(input, { target: { value: '@P' } });
			expect(queryByRole('listbox')).toBeNull();
		});

		it('dismisses dropdown on Escape', async () => {
			const { getByRole, queryByRole } = render(InputField);
			const input = getByRole('textbox') as HTMLInputElement;

			await fireEvent.input(input, { target: { value: '@P' } });
			expect(queryByRole('listbox')).toBeTruthy();

			await fireEvent.keyDown(input, { key: 'Escape' });
			expect(queryByRole('listbox')).toBeNull();
		});

		it('shows occupation for introduced NPCs', async () => {
			const { getByRole, queryAllByRole } = render(InputField);
			const input = getByRole('textbox') as HTMLInputElement;

			await fireEvent.input(input, { target: { value: '@P' } });
			const options = queryAllByRole('option');
			expect(options[0].textContent).toContain('Publican');
		});
	});
});
