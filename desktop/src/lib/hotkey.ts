import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'
import React from 'react'

export interface HotkeyConfig {
	enabled: boolean
	startRecordingHotkey: string
	stopRecordingHotkey: string
}

export interface HotkeyError {
	error: string
	needsPermission: boolean
}

class HotkeyManager {
	private unlistenHotkey: UnlistenFn | null = null
	private currentConfig: HotkeyConfig | null = null

	async registerHotkeys(config: HotkeyConfig): Promise<void> {
		try {
			await invoke('register_hotkeys', { config })
			this.currentConfig = config
			
			// Listen for hotkey events
			if (this.unlistenHotkey) {
				this.unlistenHotkey()
			}
			
			this.unlistenHotkey = await listen<string>('hotkey_triggered', (event) => {
				this.handleHotkeyTrigger(event.payload)
			})
		} catch (error: any) {
			// Handle the error appropriately
			if (error && typeof error === 'object' && 'error' in error) {
				const hotkeyError = error as HotkeyError
				throw new Error(hotkeyError.error)
			}
			throw error
		}
	}

	async unregisterHotkeys(): Promise<void> {
		try {
			await invoke('unregister_hotkeys')
			if (this.unlistenHotkey) {
				this.unlistenHotkey()
				this.unlistenHotkey = null
			}
			this.currentConfig = null
		} catch (error) {
			console.error('Failed to unregister hotkeys:', error)
			throw error
		}
	}

	async checkHotkeyAvailability(shortcut: string): Promise<boolean> {
		try {
			return await invoke('check_hotkey_availability', { shortcutStr: shortcut })
		} catch (error) {
			console.error('Failed to check hotkey availability:', error)
			return false
		}
	}

	async getRegisteredHotkeys(): Promise<Record<string, string>> {
		try {
			return await invoke('get_registered_hotkeys')
		} catch (error) {
			console.error('Failed to get registered hotkeys:', error)
			return {}
		}
	}

	private handleHotkeyTrigger(shortcutStr: string) {
		// Map shortcut string back to action based on current config
		if (!this.currentConfig) {
			return
		}
		
		let action: string | null = null
		if (shortcutStr === this.currentConfig.startRecordingHotkey) {
			action = 'start_recording'
		} else if (shortcutStr === this.currentConfig.stopRecordingHotkey) {
			action = 'stop_recording'
		}
		
		if (action) {
			// Emit custom events that the application can listen to
			const event = new CustomEvent('vibe-hotkey', {
				detail: { action }
			})
			window.dispatchEvent(event)
		}
	}

	getCurrentConfig(): HotkeyConfig | null {
		return this.currentConfig
	}

	cleanup() {
		if (this.unlistenHotkey) {
			this.unlistenHotkey()
			this.unlistenHotkey = null
		}
	}
}

// Export a singleton instance
export const hotkeyManager = new HotkeyManager()

// Hook for React components
export function useHotkeys(
	onStartRecording?: () => void,
	onStopRecording?: () => void
) {
	const handleHotkeyEvent = (event: CustomEvent<{ action: string }>) => {
		switch (event.detail.action) {
			case 'start_recording':
				onStartRecording?.()
				break
			case 'stop_recording':
				onStopRecording?.()
				break
		}
	}

	// Set up event listener
	React.useEffect(() => {
		const listener = handleHotkeyEvent as EventListener
		window.addEventListener('vibe-hotkey', listener)
		
		return () => {
			window.removeEventListener('vibe-hotkey', listener)
		}
	}, [onStartRecording, onStopRecording])

	return {
		registerHotkeys: hotkeyManager.registerHotkeys.bind(hotkeyManager),
		unregisterHotkeys: hotkeyManager.unregisterHotkeys.bind(hotkeyManager),
		checkHotkeyAvailability: hotkeyManager.checkHotkeyAvailability.bind(hotkeyManager),
		getRegisteredHotkeys: hotkeyManager.getRegisteredHotkeys.bind(hotkeyManager),
		getCurrentConfig: hotkeyManager.getCurrentConfig.bind(hotkeyManager),
	}
}