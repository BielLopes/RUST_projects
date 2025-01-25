-- Run the program
vim.keymap.set("n", "<leader>cr", function()
    vim.cmd("ToggleTerm")
    -- Send the command to the default terminal
    vim.cmd("TermExec cmd='cargo run -- /home/gabriel/Repos/RUST_projects/daemon-file-watch/src/lib.rs'")
end)
