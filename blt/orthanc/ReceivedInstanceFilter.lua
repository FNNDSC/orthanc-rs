-- Allow-list of desirable images to forward in the BLT protocol.
--
-- Documentation: https://orthanc.uclouvain.be/book/users/lua.html#filtering-incoming-dicom-instances

function ReceivedInstanceFilter(dicom, origin, info)
  if dicom.Modality == 'US' then
    return false
  else
    return true
  end
end
