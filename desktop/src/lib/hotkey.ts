import { invoke } from '@tauri-apps/api/core'
import { register, unregisterAll } from '@tauri-apps/plugin-global-shortcut'
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
	private currentConfig: HotkeyConfig | null = null
	private isRegistered = false

	async registerHotkeys(config: HotkeyConfig): Promise<void> {
		try {
			if (!config.enabled) {
				await this.unregisterHotkeys()
				return
			}

			// Unregister existing shortcuts first
			await this.unregisterHotkeys()

			// Register hotkeys using the JavaScript API
			if (config.startRecordingHotkey) {
				await register(config.startRecordingHotkey, () => {
					this.handleHotkeyTrigger(config.startRecordingHotkey)
				})
			}

			if (config.stopRecordingHotkey) {
				await register(config.stopRecordingHotkey, () => {
					this.handleHotkeyTrigger(config.stopRecordingHotkey)
				})
			}

			this.currentConfig = config
			this.isRegistered = true
			console.log('Hotkeys registered successfully')
		} catch (error: any) {
			console.error('Failed to register hotkeys:', error)
			throw new Error(`Failed to register hotkeys: ${error.message || error}`)
		}
	}

	async unregisterHotkeys(): Promise<void> {
		try {
			if (this.isRegistered) {
				await unregisterAll()
				this.isRegistered = false
				console.log('All hotkeys unregistered')
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
		this.unregisterHotkeys().catch(console.error)
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