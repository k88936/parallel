'use client';

import {useState} from 'react';
import {CreateTaskForm} from '@/components/CreateTaskForm';
import {TaskList} from '@/components/TaskList';
import type {Task} from '@/types/task';

export default function Home() {
    return (
        <div className="min-h-screen bg-gray-100">
            <div className="container mx-auto px-4 py-8 max-w-6xl">
                <header className="mb-8">
                    <h1 className="text-3xl font-bold text-gray-900">
                        Task Management System
                    </h1>
                    <p className="text-gray-600 mt-2">
                        Create and manage tasks for the GitHub agent system
                    </p>
                </header>

                <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
                    <div>
                        <CreateTaskForm/>
                    </div>
                    <div>
                        <TaskList/>
                    </div>
                </div>
            </div>
        </div>
    );
}
