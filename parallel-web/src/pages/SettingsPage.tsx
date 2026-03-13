import {useCallback, useState} from 'react';
import Heading from '@jetbrains/ring-ui-built/components/heading/heading';
import Island from '@jetbrains/ring-ui-built/components/island/island';
import IslandHeader from '@jetbrains/ring-ui-built/components/island/header';
import IslandContent from '@jetbrains/ring-ui-built/components/island/content';
import Text from '@jetbrains/ring-ui-built/components/text/text';
import Button from '@jetbrains/ring-ui-built/components/button/button';
import Checkbox from '@jetbrains/ring-ui-built/components/checkbox/checkbox';
import Select from '@jetbrains/ring-ui-built/components/select/select';
import {useAlertPreferences, type AlertType, type EventAudioConfig} from '../contexts/AlertContext';
import {AUDIO_FILES, getAudioPath, playAudioFile, type AudioSound} from '../services/audioAlerts';
import ScrollableSection from "@jetbrains/ring-ui-built/components/scrollable-section/scrollable-section";
import Group from "@jetbrains/ring-ui-built/components/group/group.js";
import Panel from "@jetbrains/ring-ui-built/components/panel/panel";

interface SoundOption {
    key: AudioSound;
    label: string;
}

interface EventConfig {
    type: AlertType;
    label: string;
    description: string;
}

const AUDIO_SOUNDS: SoundOption[] = AUDIO_FILES.map(sound => ({key: sound, label: sound.replace('.aac', '')}));

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

    const handleSoundChange = useCallback((eventType: AlertType, sound: SoundOption | null) => {
        if (sound) {
            setEventConfig(eventType, {...eventConfigs[eventType], sound: sound.key});
        }
    }, [eventConfigs, setEventConfig]);

    const handleToggle = useCallback((eventType: AlertType) => {
        setEventConfig(eventType, {...eventConfigs[eventType], enabled: !eventConfigs[eventType].enabled});
    }, [eventConfigs, setEventConfig]);

    const handlePreview = useCallback(async (eventType: AlertType) => {
        setPreviewingEvent(eventType);
        const config = eventConfigs[eventType];
        const audioPath = getAudioPath(config.sound);

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
                <ScrollableSection>
                    <Text>Configure sound effects for each event type. Each event can use a different sound and
                        volume.</Text>

                    {EVENT_CONFIGS.map(eventConfig => {
                        const config = eventConfigs[eventConfig.type];
                        const selectedSound = AUDIO_SOUNDS.find(sound => sound.key === config.sound);

                        return (
                            <Island key={eventConfig.type}>
                                <IslandHeader className="event-header">
                                    <Checkbox
                                        checked={config.enabled}
                                        onChange={() => handleToggle(eventConfig.type)}
                                    >
                                        <strong>{eventConfig.label}</strong>
                                    </Checkbox>
                                </IslandHeader>
                                <IslandContent>

                                    <Text className="event-description">{eventConfig.description}</Text>


                                    {config.enabled && (
                                        <Panel className={"gap-4"}>
                                            <Select
                                                data={AUDIO_SOUNDS}
                                                selected={selectedSound}
                                                onSelect={(sound) => handleSoundChange(eventConfig.type, sound)}
                                            />

                                            <Group>
                                                <Button
                                                    onClick={() => handlePreview(eventConfig.type)}
                                                    disabled={previewingEvent === eventConfig.type}
                                                >
                                                    {previewingEvent === eventConfig.type ? 'Playing...' : 'Preview Sound'}
                                                </Button>
                                            </Group>
                                        </Panel>
                                    )}
                                </IslandContent>
                            </Island>
                        );
                    })}

                    {/* Reset Button */}
                    <Group>
                        <Button onClick={resetToDefaults}>Reset All to Defaults</Button>
                    </Group>
                </ScrollableSection>
            </IslandContent>
        </Island>
    );
};
