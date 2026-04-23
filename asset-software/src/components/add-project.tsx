"use client";
import {FormEvent, useState} from "react";
import { invoke } from '@tauri-apps/api/core';
import { handleAddProject } from "./services";
import { useData } from "@/context/data-context";

export default function AddProjet() {
    const [name, setName] = useState("");
    const { refreshData } = useData();

    const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        try {
            await handleAddProject(name);
            await refreshData();
            setName("");
        } catch (err) {
            console.error("Failed to add project:", err);
        }
    };

    return (
        <>
            <div className=" bg-grey">

                { (
                    <form onSubmit={handleSubmit} className="flex">
                        <input
                            className="flex-1 w-26 ms-8 text-sm"
                            type="text"
                            placeholder="New project"
                            value={name}
                            onChange={(e) => setName(e.target.value)}
                            required
                        />
                            <button type="submit" className="flex-1  cursor-pointer">+</button>
                    </form>
                )}
            </div>
        </>
    );
}
