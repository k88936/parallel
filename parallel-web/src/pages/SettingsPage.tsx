import {useCallback, useState} from 'react';
import Heading from '@jetbrains/ring-ui-built/components/heading/heading';
import Island from '@jetbrains/ring-ui-built/components/island/island';
import IslandHeader from '@jetbrains/ring-ui-built/components/island/header';
import IslandContent from '@jetbrains/ring-ui-built/components/island/content';
import Text from '@jetbrains/ring-ui-built/components/text/text';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import Checkbox from '@jetbrains/ring-ui-built/components/checkbox/checkbox';
import Select from '@jetbrains/ring-ui-built/components/select/select';
import {useAlertPreferences, type AlertType, type AudioCategory, type EventAudioConfig} from '../contexts/AlertContext';
import {getRandomAudioFile, getAudioPath, playAudioFile} from '../services/audioAlerts';
import './SettingsPage.css';

interface CategoryOption {
    key: AudioCategory;
    label: string;
}

interface EventConfig {
    type: AlertType;
    label: string;
    description: string;
}

const AUDIO_CATEGORIES: CategoryOption[] = [
    {key: 'alert', label: 'Alert (10 variations)'},
    {key: 'nope', label: 'Nope (12 variations)'},
    {key: 'yup', label: 'Yup (6 variations)'},
    {key: 'bip-bop', label: 'Bip-Bop (10 variations)'},
    {key: 'staplebops', label: 'Staplebops (7 variations)'},
];

const EVENT_CONFIGS: EventConfig[] = [
    {type: 'worker_offline', label: 'Worker Offline', description: 'When a worker goes offline unexpectedly'},
    {type: 'worker_online', label: 'Worker Online', description: 'When a worker comes back online'},
    {type: 'task_timeout', label: 'Task Timeout', description: 'When a task exceeds max execution time'},
    {type: 'task_review_requested', label: 'Task Review Requested', description: 'When a task requires human review'},
    {type: 'task_completed', label: 'Task Completed', description: 'When a task completes successfully'},
    {type: 'task_failed', label: 'Task Failed', description: 'When a task fails'},
    {type: 'task_cancelled', label: 'Task Cancelled', description: 'When a task is cancelled'},
];

export const SettingsPage = () => {
    const {
        worker_offline,
        worker_online,
        task_timeout,
        task_review_requested,
        task_completed,
        task_failed,
        task_cancelled,
        setEventConfig,
        resetToDefaults,
    } = useAlertPreferences();

    const [previewingEvent, setPreviewingEvent] = useState<AlertType | null>(null);

    const eventConfigs: Record<AlertType, EventAudioConfig> = {
        worker_offline,
        worker_online,
        task_timeout,
        task_review_requested,
        task_completed,
        task_failed,
        task_cancelled,
    };

    const handleCategoryChange = useCallback((eventType: AlertType, category: CategoryOption | null) => {
        if (category) {
            setEventConfig(eventType, {...eventConfigs[eventType], category: category.key});
        }
    }, [eventConfigs, setEventConfig]);

    const handleToggle = useCallback((eventType: AlertType) => {
        setEventConfig(eventType, {...eventConfigs[eventType], enabled: !eventConfigs[eventType].enabled});
    }, [eventConfigs, setEventConfig]);

    const handleVolumeChange = useCallback((eventType: AlertType, volume: number) => {
        setEventConfig(eventType, {...eventConfigs[eventType], volume});
    }, [eventConfigs, setEventConfig]);

    const handlePreview = useCallback(async (eventType: AlertType) => {
        setPreviewingEvent(eventType);
        const config = eventConfigs[eventType];
        const audioFile = getRandomAudioFile(config.category);
        const audioPath = getAudioPath(audioFile);
        
        try {
            await playAudioFile(audioPath, config.volume);
        } catch (err) {
            console.error('Preview failed:', err);
        } finally {
            setPreviewingEvent(null);
        }
    }, [eventConfigs]);

    return (
        <Island>
            <IslandHeader border>
                <Heading level={1}>Alert Sound Settings</Heading>
            </IslandHeader>
            <IslandContent>
                <div className="settings-container">
                    <Text>Configure sound effects for each event type. Each event can use a different sound category and volume.</Text>

                    {EVENT_CONFIGS.map(eventConfig => {
                        const config = eventConfigs[eventConfig.type];
                        const selectedCategory = AUDIO_CATEGORIES.find(cat => cat.key === config.category);

                        return (
                            <div key={eventConfig.type} className="settings-section event-section">
                                <div className="event-header">
                                    <Checkbox
                                        checked={config.enabled}
                                        onChange={() => handleToggle(eventConfig.type)}
                                    >
                                        <strong>{eventConfig.label}</strong>
                                    </Checkbox>
                                </div>
                                <p className="event-description">{eventConfig.description}</p>

                                {config.enabled && (
                                    <div className="event-controls">
                                        <div className="settings-option">
                                            <label className="settings-label">Sound Category</label>
                                            <Select
                                                data={AUDIO_CATEGORIES}
                                                selected={selectedCategory}
                                                onSelect={(cat) => handleCategoryChange(eventConfig.type, cat)}
                                            />
                                        </div>

                                        <div className="settings-option">
                                            <label className="settings-label">
                                                Volume: {(config.volume * 100).toFixed(0)}%
                                            </label>
                                            <input
                                                type="range"
                                                min="0"
                                                max="1"
                                                step="0.1"
                                                value={config.volume}
                                                onChange={(e) => handleVolumeChange(eventConfig.type, parseFloat(e.target.value))}
                                                className="settings-slider"
                                            />
                                        </div>

                                        <div className="settings-option">
                                            <Button
                                                onClick={() => handlePreview(eventConfig.type)}
                                                disabled={previewingEvent === eventConfig.type}
                                            >
                                                {previewingEvent === eventConfig.type ? 'Playing...' : 'Preview Sound'}
                                            </Button>
                                        </div>
                                    </div>
                                )}
                            </div>
                        );
                    })}

                    {/* Reset Button */}
                    <div className="settings-section settings-actions">
                        <Button onClick={resetToDefaults}>Reset All to Defaults</Button>
                    </div>
                </div>
            </IslandContent>
        </Island>
    );
};
