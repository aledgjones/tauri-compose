#include "public.sdk/source/vst/hosting/hostclasses.h"

extern "C" Steinberg::Vst::IHostApplication* solo_compose_create_steinberg_host_application ()
{
    return new Steinberg::Vst::HostApplication ();
}

extern "C" void solo_compose_destroy_steinberg_host_application (
    Steinberg::Vst::IHostApplication* host_application)
{
    delete static_cast<Steinberg::Vst::HostApplication*> (host_application);
}