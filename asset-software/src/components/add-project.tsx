"use client";
import {FormEvent, useState} from "react";
import { invoke } from '@tauri-apps/api/core';
import { handleAddProject } from "./services";

export default function AddProjet() {
    const [name, setName] = useState("");

    const handleSubmit = async (e: FormEvent<HTMLFormElement>) => {
        e.preventDefault();
        let name = e.target[0].value
        await handleAddProject(name);
        setName("");
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
