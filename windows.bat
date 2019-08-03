set VK_ICD_FILENAMES=%CMD%\libportability-icd\portability-win-debug.json
cd ..\VK-GL-CTS\build\external\vulkancts\modules\vulkan
Debug\deqp-vk.exe dEQP-VK.draw.*
