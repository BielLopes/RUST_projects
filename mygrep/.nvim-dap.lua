-- Run the program
vim.keymap.set("n", "<leader>cr", function()
    vim.cmd("ToggleTerm")
    local query = vim.fn.input("Enter the shearch query: ")
    -- Send the command to the default terminal
    vim.cmd("TermExec cmd='cargo run -- " .. query .. " poem.txt'")
end)

-- Runt tests
vim.keymap.set("n", "<leader>ct", function()
    vim.cmd("ToggleTerm")
    -- Send the command to the default terminal
    vim.cmd("TermExec cmd='cargo test'")
end)

local dap = require("dap")

dap.adapters.lldb = {
    type = "executable",
    command = "/usr/bin/lldb-dap",
    name = "lldb",
}

dap.configurations.rust = {
    {
        name = "mygrep",
        type = "lldb",
        request = "launch",
        program = function()
            return vim.fn.getcwd() .. "/target/debug/mygrep"
        end,
        cwd = "${workspaceFolder}",
        runInTerminal = false,
        stopOnEntry = false,
        args = {"text", "poem.txt"},
    },
}
